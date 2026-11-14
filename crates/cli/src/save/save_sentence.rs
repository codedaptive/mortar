//! save_sentence: the one sentence `mortar save` takes, checked before anything runs. The one-argument check is
//! the verb's own; the blank and line-break checks are snapshot's sentence_refusal (S-41, exported per S-90).
//!
//! Nagatha's RITs (S-101) in crates/cli/tests/save_sentence_tests.rs join this CLI check to
//! snapshot::sentence_refusal: one_sentence_is_the_word, an_empty_sentence_is_refused,
//! a_whitespace_sentence_is_refused, a_newline_is_refused and a_carriage_return_is_refused.
//! They check the returned sentence, snapshot's blank/line-break reasons and each refusal's exit 1,
//! "nothing ran" and next `mortar save "<one line saying what changed>"`.
//! That tests file also holds the ZITs no_word_is_refused and two_words_are_refused for the one-argument check;
//! they do not call snapshot's check and run only the cli's own save_error_report.
//! No sibling save_sentence_tests.rs (S-104): every test of this file is a ZIT or a RIT.

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
