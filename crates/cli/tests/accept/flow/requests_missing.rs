//! Which requests a squash commit's message does not name (check 7).

/// The squash message names a request with a line `- <request>: <subjects>` (land's squash_message). Returns, in
/// the order given, each of `requests` that has no such line.
pub fn requests_missing(message: &str, requests: &[&str]) -> Vec<String> {
    requests
        .iter()
        .filter(|request| !message.lines().any(|line| line.starts_with(&format!("- {request}: "))))
        .map(|request| request.to_string())
        .collect()
}
