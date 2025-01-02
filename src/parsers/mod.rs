use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use ip_filtering::filter_ips;

mod game;
mod ip_filtering;
mod runtimes;

// Ideally this would take in the date for versioning, but specifically the date of the most recent commit?
// I want to make sure that we can essentially pinpoint exactly when a log change happens.
// TODO: IP filter every file, replace the read_to_string's with them
#[tracing::instrument(skip_all)]
pub fn parse_file(path: &Path) -> eyre::Result<Option<(Cow<'static, Path>, String)>> {
    let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
        return Ok(None);
    };

    match filename {
        "game.log" => Ok(Some((
            Cow::Borrowed(Path::new("game.txt")),
            game::process_game_log(&filter_ips(&read_to_string(path)?)),
        ))),

        // Runtime condensing is done in the runtimes.rs parser
        "runtime.log" => Ok(Some((
            Cow::Borrowed(Path::new("runtime.txt")),
            runtimes::process_runtimes_log(&filter_ips(&read_to_string(path)?)),
        ))),

        // Pass through, but replace .txt with .log
        "asset.log"
        | "attack.log"
        | "cloning.log"
        | "dynamic.log"
        | "econ.log"
        | "economy.log"
        | "harddel.log"
        | "harddels.log"
        | "initialize.log"
        | "job_debug.log"
        | "manifest.log"
        | "map_errors.log"
        | "mecha.log"
        | "mob_tags.log"
        | "overlay.log"
        | "paper.log"
        | "pda.log"
        | "qdel.log"
        | "shuttle.log"
        | "signal.log"
        | "signals.log"
        | "silicon.log"
        | "silo.log"
        | "speech_indicators.log"
        | "telecomms.log"
        | "tool.log"
        | "tools.log"
        | "uplink.log"
        | "virus.log" => Ok(Some((
            Cow::Owned(PathBuf::from(filename).with_extension("txt")),
            read_to_string(path)?,
        ))),

        // Pass through, including with filename
        "asset.log.json"
        | "attack.log.json"
        | "atmos.html"
        | "botany.html"
        | "cargo.html"
        | "circuit.html"
        | "cloning.log.json"
        | "crafting.html"
        | "deaths.html"
        | "dynamic.json"
        | "dynamic.log.json"
        | "econ.log.json"
        | "economy.log.json"
        | "engine.html"
        | "experimentor.html"
        | "gravity.html"
        | "hallucinations.html"
        | "harddel.log.json"
        | "harddels.log.json"
        | "hypertorus.html"
        | "id_card_changes.html"
        | "init_profiler.json"
        | "init_times.json"
        | "initialize.log.json"
        | "job_debug.log.json"
        | "kudzu.html"
        | "manifest.log.json"
        | "map_errors.log.json"
        | "mecha.log.json"
        | "mob_tags.log.json"
        | "nanites.html"
        | "newscaster.json"
        | "overlay.log.json"
        | "paper.log.json"
        | "pda.log.json"
        | "portals.html"
        | "presents.html"
        | "profiler.json"
        | "qdel.log.json"
        | "radiation.html"
        | "records.html"
        | "research.html"
        | "round_end_data.html"
        | "round_end_data.json"
        | "sendmaps.json"
        | "shuttle.log.json"
        | "signal.log.json"
        | "signals.log.json"
        | "silicon.log.json"
        | "silo.json"
        | "silo.log.json"
        | "singulo.html"
        | "speech_indicators.log.json"
        | "supermatter.html"
        | "target_zone_switch.json"
        | "telecomms.log.json"
        | "telesci.html"
        | "tool.log.json"
        | "tools.log.json"
        | "uplink.log.json"
        | "virus.log.json"
        | "wires.html" => Ok(Some((
            Cow::Owned(PathBuf::from(filename)),
            read_to_string(path)?,
        ))),

        perf_filename if perf_filename.starts_with("perf-") => Ok(Some((
            Cow::Owned(PathBuf::from(filename)),
            read_to_string(path)?,
        ))),

        _ => Ok(None),
    }
}

// Separate so we can tracy it
#[tracing::instrument(skip_all)]
fn read_to_string(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn test_log_directory(log_directory: &Path, public_log_directory: &Path) {
        let mut inequals = Vec::new();
        let mut filenames_unseen = HashSet::new();

        for day_folder_entry in std::fs::read_dir(log_directory).unwrap() {
            let day_folder_entry = day_folder_entry.unwrap();

            for round_entry in std::fs::read_dir(day_folder_entry.path()).unwrap() {
                let round_entry = round_entry.unwrap();

                // A couple broken logs
                if round_entry.file_name() == "round-197406" {
                    continue;
                }

                let php_parsed_folder = public_log_directory
                    .join(day_folder_entry.file_name())
                    .join(round_entry.file_name());

                let mut php_parsed_files_unseen = HashSet::new();

                for php_entry in std::fs::read_dir(&php_parsed_folder).unwrap() {
                    let php_entry = php_entry.unwrap();
                    php_parsed_files_unseen.insert(php_entry.file_name());
                }

                for log_entry in std::fs::read_dir(round_entry.path()).unwrap() {
                    let log_entry = log_entry.unwrap();
                    let log_stem = log_entry
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();

                    match parse_file(&log_entry.path()) {
                        Ok(Some((filename, parsed))) => {
                            if !php_parsed_files_unseen.remove(filename.file_name().unwrap()) {
                                panic!(
                                    "{} wasn't in php_parsed_files of {}",
                                    filename.file_name().unwrap().to_string_lossy(),
                                    round_entry.path().display()
                                );
                            }

                            let php_parsed_path = php_parsed_folder.join(&filename);
                            assert!(php_parsed_path.exists());

                            let raw_lines = read_to_string(&log_entry.path())
                                .unwrap()
                                .lines()
                                .map(|line| line.to_owned())
                                .collect::<Vec<_>>();

                            let rust_lines = parsed
                                .lines()
                                .map(|line| line.to_owned())
                                .collect::<Vec<_>>();

                            let php_lines = read_to_string(&php_parsed_path)
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
                                    if log_stem == "game" {
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
                                    }

                                    inequals.push((
                                        log_entry.path().to_string_lossy().into_owned(),
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

                filenames_unseen.extend(
                    php_parsed_files_unseen
                        .into_iter()
                        // sql.txt got moved over, and older parsed versions of it were totally broken
                        .filter(|filename| filename != "sql.txt")
                        // I don't know why HTML parsing is a thing, and it's a little complicated
                        .filter(|filename| filename != "game.html")
                        // Gonna be handled by the routing itself
                        .filter(|filename| filename != "runtime.condensed.txt")
                        .filter(|filename| filename != "photos"),
                );
            }
        }

        for (filename, raw_line, rust_example, php_example) in inequals.iter().take(10) {
            println!(
                "filename:{filename}\nraw: {raw_line}\nrust: {rust_example}\nphp: {php_example}\n"
            );
        }

        if !inequals.is_empty() {
            panic!("{} lines didn't match", inequals.len());
        }

        assert_eq!(filenames_unseen, HashSet::new());
    }

    #[test]
    fn test_2023_01_logs() {
        crate::tracy::enable_tracy();

        test_log_directory(
            Path::new("raw-logs-tests/sybil-2023-01"),
            Path::new("raw-logs-tests/sybil-2023-01-public"),
        );
    }

    #[test]
    fn test_2023_11_logs() {
        crate::tracy::enable_tracy();

        test_log_directory(
            Path::new("raw-logs-tests/sybil-2023-11"),
            Path::new("raw-logs-tests/sybil-2023-11-public"),
        );
    }
}
