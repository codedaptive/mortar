//! save_message: a save commit's message before its trailers.

use crate::SnapshotError;

/// `sentence` as the subject line, ending in a newline, kept as given. An empty or whitespace-only sentence, or
/// one holding a line break ('\n' or '\r'), gives BadSentence: a subject is one line with text on it.
pub(crate) fn save_message(sentence: &str) -> Result<String, SnapshotError> {
    let reason = if sentence.trim().is_empty() {
        "the sentence is empty or only whitespace"
    } else if sentence.contains(['\n', '\r']) {
        "the sentence holds a line break; a commit subject is one line"
    } else {
        return Ok(format!("{sentence}\n"));
    };
    Err(SnapshotError::BadSentence { reason: reason.to_string() })
}
