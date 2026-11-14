//! save_sentence: the one sentence `mortar save` takes, checked before anything runs. A stopgap copy of the
//! blank and line-break checks of snapshot's private save_message.rs (S-41); the cli drops it when WO-snapshot-8
//! exports the check (S-90).

use snapshot::SnapshotError;

use super::save_error_report;
use crate::report::Report;

/// The one word of `rest` when it is a sentence a save can carry: one argument, not empty or whitespace-only,
/// holding no '\n' or '\r'. The blank and line-break checks are S-41's, copied from snapshot's save_message;
/// the one-argument check is the verb's own. The cli checks all first so a refused sentence runs no formatter
/// and takes no lock. Every other case is refused (exit 1), saying why,
/// as save_error_report reports a BadSentence.
pub fn save_sentence(rest: &[String]) -> Result<&str, Report> {
    let reason = match rest {
        [sentence] if sentence.trim().is_empty() => "the sentence is empty or only whitespace",
        [sentence] if sentence.contains(['\n', '\r']) => {
            "the sentence holds a line break; a save's sentence is one line (S-41)"
        }
        [sentence] => return Ok(sentence),
        _ => "save takes one sentence, quoted as one argument",
    };
    Err(save_error_report(&SnapshotError::BadSentence { reason: reason.to_string() }))
}
