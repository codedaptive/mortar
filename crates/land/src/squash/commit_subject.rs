//! commit_subject: a commit's subject like git's `%s`, with each line also trimmed at both ends.

/// The first paragraph of `message` (leading blank lines skipped, ended by the first blank line), each line
/// trimmed and the lines joined by one space, so a subject wrapped over several lines reads as one.
pub(crate) fn commit_subject(message: &str) -> String {
    message
        .lines()
        .map(str::trim)
        .skip_while(|line| line.is_empty())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
