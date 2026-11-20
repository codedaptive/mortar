//! save_message: a save commit's message before its trailers.

use crate::sentence_refusal::sentence_refusal;
use crate::SnapshotError;

/// `sentence` as the subject line, ending in a newline, kept as given. A sentence `sentence_refusal` refuses
/// (empty or whitespace-only, or holding a line break) gives BadSentence with that refusal's reason.
pub(crate) fn save_message(sentence: &str) -> Result<String, SnapshotError> {
    match sentence_refusal(sentence) {
        Some(reason) => Err(SnapshotError::BadSentence { reason: reason.to_string() }),
        None => Ok(format!("{sentence}\n")),
    }
}
