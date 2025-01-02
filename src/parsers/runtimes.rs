use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;

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
