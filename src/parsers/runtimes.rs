use std::{borrow::Cow, collections::HashMap, iter::Peekable, sync::LazyLock};

use regex::Regex;

use crate::parsers::ip_filtering::filter_ips;

pub fn process_runtimes_log(contents: &str) -> String {
    contents
        .lines()
        .map(|line| sanitize_runtimes_line(line))
        .collect::<Vec<_>>()
        .join("\n")
}

// Remove BYOND printed strings
fn sanitize_runtimes_line(line: &str) -> Cow<str> {
    static STRING_OUTPUT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"^.*Cannot read ".*$"#).unwrap());

    STRING_OUTPUT_REGEX.replace(line, "-censored (string output)")
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct CondensedRuntimeKey<'a> {
    message: &'a str,
    proc_name: &'a str,
}

#[derive(Debug)]
struct CondensedRuntimeValue<'a> {
    source_file: Option<&'a str>,
    usr: &'a str,
    src: &'a str,
    src_loc: Option<&'a str>,

    count: u64,
}

pub fn condense_runtimes(contents: &str) -> String {
    let contents = filter_ips(contents);

    let condensed_runtimes = get_condensed_runtimes(&contents);

    let mut condensed_runtimes_sorted: Vec<(CondensedRuntimeKey<'_>, CondensedRuntimeValue<'_>)> =
        condensed_runtimes.runtimes.into_iter().collect::<Vec<_>>();
    condensed_runtimes_sorted.sort_by_key(|(_, runtime)| u64::MAX - runtime.count);

    let mut lines = vec![
		"Note: The source file, src and usr are all from the FIRST of the identical runtimes. Everything else is cropped.".to_owned(),
		"".to_owned(),
		format!("Total unique runtimes: {}", condensed_runtimes_sorted.len()),
		format!("Total runtimes: {}", condensed_runtimes.total_count),
		"".to_owned(),
		"** Runtimes **".to_owned(),
	];

    for (key, value) in condensed_runtimes_sorted {
        lines.push("".to_owned());

        lines.push(format!(
            "The following runtime has occurred {} time(s).",
            value.count
        ));

        lines.push(format!("runtime error: {}", key.message));
        lines.push(format!("proc name: {}", key.proc_name));

        if let Some(source_file) = value.source_file {
            lines.push(format!("  source file: {source_file}"));
        }

        lines.push(format!("  usr: {}", value.usr));
        lines.push(format!("  src: {}", value.src));

        if let Some(src_loc) = value.src_loc {
            lines.push(format!("  src.loc: {src_loc}"));
        }

        lines.push("".to_owned());
    }

    lines.push("".to_owned());
    lines.push("".to_owned());

    lines.join("\n")
}

struct CondensedRuntimes<'a> {
    total_count: u64,
    runtimes: HashMap<CondensedRuntimeKey<'a>, CondensedRuntimeValue<'a>>,
}

fn get_condensed_runtimes(runtime_contents: &str) -> CondensedRuntimes {
    let mut lines = runtime_contents.lines().peekable();
    let mut condensed_runtimes: HashMap<CondensedRuntimeKey, CondensedRuntimeValue> =
        HashMap::new();

    static RE_RUNTIME_ERROR_START: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\[.+?\] (?:RUNTIME: )?runtime error: (.*)$").unwrap());

    static RE_RUNTIME_PROC_NAME: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^ \- (?:proc|verb) name: (.+)$").unwrap());

    let mut runtime_count = 0;

    'main_loop: while let Some(start_line) = lines.next() {
        let Some(runtime_error_start) = RE_RUNTIME_ERROR_START.captures(start_line) else {
            continue;
        };

        let runtime = runtime_error_start.get(1).unwrap();
        runtime_count += 1;

        // Some runtimes are multi-line
        let proc_name = loop {
            match lines.next() {
                Some(next_line) => match RE_RUNTIME_PROC_NAME.captures(next_line) {
                    Some(proc_name) => break proc_name,
                    None => continue,
                },

                None => {
                    break 'main_loop;
                }
            }
        };

        let condensed_runtime_key = CondensedRuntimeKey {
            message: runtime.as_str(),
            proc_name: proc_name.get(1).unwrap().as_str(),
        };

        if let Some(condensed_runtime_value) = condensed_runtimes.get_mut(&condensed_runtime_key) {
            condensed_runtime_value.count += 1;
            continue;
        }

        let source_file = read_field(&mut lines, "source file");

        let Some(usr) = read_field(&mut lines, "usr") else {
            tracing::error!("next line was not usr");
            continue;
        };

        let Some(src) = read_field(&mut lines, "src") else {
            tracing::error!("next line was not src");
            continue;
        };

        // It's okay to not have this despite moving the line, since we're not going to get a relevant one anyway
        let src_loc = read_field(&mut lines, "src.loc");

        condensed_runtimes.insert(
            condensed_runtime_key,
            CondensedRuntimeValue {
                source_file,
                usr,
                src,
                src_loc,
                count: 1,
            },
        );
    }

    CondensedRuntimes {
        total_count: runtime_count,
        runtimes: condensed_runtimes,
    }
}

fn read_field<'a>(
    peekable_lines: &mut Peekable<impl Iterator<Item = &'a str>>,
    expecting: &'static str,
) -> Option<&'a str> {
    static RE_RUNTIME_FIELD: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^ \-   (.+?): (.+)$").unwrap());

    let line = peekable_lines.peek()?;

    let capture = RE_RUNTIME_FIELD.captures(line)?;
    let field_captured = capture.get(1).unwrap();
    if field_captured.as_str() != expecting {
        return None;
    }

    peekable_lines.next();
    let value = capture.get(2).unwrap();
    Some(value.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // This implementation fixes some bugs in the C++ implementation
    fn similar(rust_section: &str, cpp_section: &str) -> bool {
        if rust_section == cpp_section {
            return true;
        }

        // This runtime has multiple lines, which C++ can't handle
        if rust_section.contains("Use override = TRUE to suppress this warning.") {
            return cpp_section.contains("Use override = TRUE to suppress this warning.");
        }

        if rust_section.contains("refusing to update and cutting.") {
            return cpp_section.contains("refusing to update and cutting.");
        }

        // C++ handles empty runtimes differently
        if rust_section.contains("runtime error: \n") {
            return cpp_section.contains("runtime error: proc name:");
        }

        // C++ doesn't handle verbs
        if rust_section.contains("/verb/") {
            return !cpp_section.contains("proc name");
        }

        false
    }

    fn test_log_directory(log_directory: &Path, public_log_directory: &Path) {
        if !log_directory.exists() {
            return;
        }

        for day_folder_entry in std::fs::read_dir(log_directory).unwrap() {
            let day_folder_entry = day_folder_entry.unwrap();

            for round_entry in std::fs::read_dir(day_folder_entry.path()).unwrap() {
                let round_entry = round_entry.unwrap();

                // A couple broken logs
                if round_entry.file_name() == "round-197406" {
                    continue;
                }

                let raw_runtimes_path = round_entry.path().join("runtime.log");
                if !raw_runtimes_path.exists() {
                    continue;
                }

                let raw_runtimes = std::fs::read_to_string(raw_runtimes_path).unwrap();

                let cpp_parsed_condensed = std::fs::read_to_string(
                    public_log_directory
                        .join(day_folder_entry.file_name())
                        .join(round_entry.file_name())
                        .join("runtime.condensed.txt"),
                )
                .unwrap();

                let condensed_runtimes = condense_runtimes(&raw_runtimes);

                // The C++ runtime condenser only sorts by count, which means everything else is unspecified.
                let mut rust_split = condensed_runtimes
                    .split("\n\n")
                    .map(str::trim)
                    .collect::<Vec<_>>();
                rust_split.sort();

                let mut cpp_split = cpp_parsed_condensed
                    .split("\n\n")
                    .map(str::trim)
                    .collect::<Vec<_>>();
                cpp_split.sort_by_key(|str| {
                    str.replace("runtime error: proc name:", "runtime error: \nproc name:")
                });

                for (&rust, &cpp) in rust_split.iter().zip(&cpp_split) {
                    if !similar(rust, cpp) {
                        panic!(
                            "round: {}\n\nrust: {rust}\n\ncpp: {cpp}",
                            round_entry.path().display()
                        );
                    }
                }

                if rust_split.len() != cpp_split.len() {
                    panic!(
                        "{} didn't match sections ({} vs. {})",
                        round_entry.path().display(),
                        rust_split.len(),
                        cpp_split.len()
                    );
                }
            }
        }
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
