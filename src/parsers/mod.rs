use std::{ffi::OsStr, path::Path};

mod game;

// Ideally this would take in the date for versioning, but specifically the date of the most recent commit?
// I want to make sure that we can essentially pinpoint exactly when a log change happens.
pub fn parse_file(path: &Path) -> eyre::Result<Option<String>> {
    match path.file_name().and_then(OsStr::to_str) {
        Some("game.log") => Ok(Some(game::process_game_log(&std::fs::read_to_string(
            path,
        )?))),

        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_log_directory(log_directory: &Path, public_log_directory: &Path) {
        let mut inequals = Vec::new();

        for day_folder_entry in std::fs::read_dir(log_directory).unwrap() {
            let day_folder_entry = day_folder_entry.unwrap();
            for round_entry in std::fs::read_dir(day_folder_entry.path()).unwrap() {
                let round_entry = round_entry.unwrap();
                for log_entry in std::fs::read_dir(round_entry.path()).unwrap() {
                    let log_entry = log_entry.unwrap();
                    let log_stem = log_entry
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();

                    if log_stem != "game" {
                        continue;
                    }

                    let php_parsed_path = public_log_directory
                        .join(day_folder_entry.file_name())
                        .join(round_entry.file_name())
                        .join(log_entry.path().with_extension("txt").file_name().unwrap());
                    assert!(php_parsed_path.exists());

                    match parse_file(&log_entry.path()) {
                        Ok(Some(parsed)) => {
                            let raw_lines = std::fs::read_to_string(log_entry.path())
                                .unwrap()
                                .lines()
                                .map(|line| line.to_owned())
                                .collect::<Vec<_>>();

                            let rust_lines = parsed
                                .lines()
                                .map(|line| line.to_owned())
                                .collect::<Vec<_>>();

                            let php_lines = std::fs::read_to_string(php_parsed_path)
                                .unwrap()
                                .lines()
                                .map(|line| line.to_owned())
                                .collect::<Vec<_>>();

                            if rust_lines.len() != php_lines.len() {
                                panic!(
                                    "{} needed {} lines, but has {}",
                                    log_entry.path().display(),
                                    php_lines.len(),
                                    rust_lines.len()
                                );
                            }

                            for (raw_line, (rust_line, php_line)) in raw_lines
                                .iter()
                                .zip(rust_lines.iter().zip(php_lines.iter()))
                            {
                                if rust_line != php_line {
                                    // PHP bugs
                                    if (rust_line.contains("GAME-INTERNET-REQUEST")
                                        || rust_line.contains("GAME-RADIO-EMOTE"))
                                        && php_line.contains("no_category_colon")
                                    {
                                        continue;
                                    }

                                    if php_line.contains("-censored(asay/apm/ahelp)-")
                                        && rust_line.contains("ahelp/notes/etc")
                                    {
                                        continue;
                                    }

                                    // Changes made since 2022
                                    if php_line.contains("censored(misc)")
                                        && rust_line.contains("censored(no_ts_start)")
                                    {
                                        continue;
                                    }

                                    if php_line.contains("no_category_colon")
                                        && rust_line.contains("Starting up round ID")
                                    {
                                        continue;
                                    }

                                    inequals.push((
                                        raw_line.to_owned(),
                                        rust_line.to_owned(),
                                        php_line.to_owned(),
                                    ));
                                }
                            }
                        }

                        Ok(None) => {}

                        Err(error) => {
                            panic!("couldn't parse {}: {error:?}", log_entry.path().display());
                        }
                    }
                }
            }
        }

        for (raw_line, rust_example, php_example) in inequals.iter().take(10) {
            println!("raw: {raw_line}\nrust: {rust_example}\nphp: {php_example}\n");
        }

        if !inequals.is_empty() {
            panic!("{} lines didn't match", inequals.len());
        }
    }

    #[test]
    fn test_2023_logs() {
        test_log_directory(
            Path::new("raw-logs-tests/sybil-2023-11"),
            Path::new("raw-logs-tests/sybil-2023-11-public"),
        );
    }
}
