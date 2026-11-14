//! read_remote_push_urls: read the answer of `git remote get-url --push --all`.

use crate::text_refusal::text_refusal;
use crate::GitError;

/// One URL per `\n`-ended line, in git's order, each kept exactly as printed: nothing is trimmed, so a CR before the
/// line's LF stays in the URL. Empty output is no URLs. Output without its final newline is refused naming the text.
/// Output that is not UTF-8 is refused naming its lossy form, never returned lossily, because callers compare the
/// exact text.
pub(crate) fn read_remote_push_urls(stdout: &[u8]) -> Result<Vec<String>, GitError> {
    let text = std::str::from_utf8(stdout)
        .map_err(|_| text_refusal("remote URLs that are not UTF-8", &String::from_utf8_lossy(stdout)))?;
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let body = text
        .strip_suffix('\n')
        .ok_or_else(|| text_refusal("remote URLs without their final newline", text))?;
    Ok(body.split('\n').map(str::to_string).collect())
}
