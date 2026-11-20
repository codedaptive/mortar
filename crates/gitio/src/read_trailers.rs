//! read_trailers: read `interpret-trailers --parse` output as (key, value) pairs.

use crate::read_message::read_message;
use crate::GitError;

/// One pair per `Key: value` line, in git's order, split at the first `:` with one leading space of the value
/// dropped; git has already unfolded folded values. Empty output is no trailers. Output that is not UTF-8, or a
/// line without `:`, is refused naming the text.
pub(crate) fn read_trailers(stdout: &[u8]) -> Result<Vec<(String, String)>, GitError> {
    read_message(stdout)?
        .split_terminator('\n')
        .map(|line| {
            let (key, value) = line.split_once(':').ok_or_else(|| {
                GitError::refusal(Vec::new(), format!("trailer line without ':': {line:?}"))
            })?;
            Ok((key.to_string(), value.strip_prefix(' ').unwrap_or(value).to_string()))
        })
        .collect()
}
