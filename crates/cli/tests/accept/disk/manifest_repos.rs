//! Reads the repository paths out of one team manifest.

/// The quoted value of every `repo = "<path>"` line, in file order with duplicates kept. The key may be indented
/// and spaced from `=`; the value runs from its opening quote to the next one, so a trailing comment is fine. A
/// commented-out line, another key (`repository`, `repo_url`), an unquoted value or an unclosed quote is skipped.
/// The manifest is read as text, not parsed as TOML, so the drive needs no TOML reader.
pub fn manifest_repos(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let value = line.trim_start().strip_prefix("repo")?.trim_start_matches(' ').strip_prefix('=')?;
            let quoted = value.trim_start_matches(' ').strip_prefix('"')?;
            quoted.split_once('"').map(|(path, _)| path.to_string())
        })
        .collect()
}
