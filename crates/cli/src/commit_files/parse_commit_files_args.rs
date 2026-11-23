//! The words of `mr commit-files` after its globals, read into the paths it commits and the caller's message.
//! parse_commit_files_args_tests covers it.

use std::path::PathBuf;

/// Each word is a path (as given), except at most one `--message <text>` or `--message=<text>`, before or after the
/// paths; the text is kept as given. The Err is the reason, the first in line order: --message with no word after
/// it, a second --message, a text empty or only whitespace, any other word starting with "-" (named); then, at the
/// end, no path.
pub fn parse_commit_files_args(rest: &[String]) -> Result<(Vec<PathBuf>, Option<String>), String> {
    let mut paths = Vec::new();
    let mut message: Option<String> = None;
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        let text = match word.strip_prefix("--message=") {
            Some(text) => text,
            None if word == "--message" => words.next().map(String::as_str).ok_or("--message needs a text")?,
            None if word.starts_with('-') => return Err(format!("unknown option {word:?}")),
            None => {
                paths.push(PathBuf::from(word));
                continue;
            }
        };
        if message.is_some() {
            return Err("--message is given twice".to_string());
        }
        if text.trim().is_empty() {
            return Err("--message needs a text that is not empty or only whitespace".to_string());
        }
        message = Some(text.to_string());
    }
    if paths.is_empty() {
        return Err("commit-files needs at least one path".to_string());
    }
    Ok((paths, message))
}
