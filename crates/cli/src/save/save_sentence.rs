//! save_sentence: the one sentence `mr save` takes, checked before anything runs. The one-argument check is
//! the verb's own; the blank and line-break checks are snapshot's sentence_refusal, which snapshot exports.
//! save_sentence_tests covers the one-argument check (no word, two words) and the one-argument cases, which run
//! snapshot::sentence_refusal (exported and pure on the text, so a unit test may use it): the returned sentence,
//! snapshot's blank and line-break reasons, and each refusal's exit 1, "nothing ran" and next
//! `mr save "<one line saying what changed>"`.

use snapshot::{sentence_refusal, SnapshotError};

use super::save_error_report;
use crate::report::Report;

/// The one word of `rest` when it is a sentence a save can carry. Exactly one argument is the verb's own rule;
/// that argument is then asked of snapshot's sentence_refusal, which refuses a blank or multi-line sentence.
/// The cli checks first so a refused sentence runs no formatter and takes no lock. Every refusal (exit 1) says
/// why, as save_error_report reports a BadSentence.
pub fn save_sentence(rest: &[String]) -> Result<&str, Report> {
    let reason = match rest {
        [sentence] => match sentence_refusal(sentence) {
            None => return Ok(sentence),
            Some(reason) => reason,
        },
        _ => "save takes one sentence, quoted as one argument",
    };
    Err(save_error_report(&SnapshotError::BadSentence { reason: reason.to_string() }))
}
