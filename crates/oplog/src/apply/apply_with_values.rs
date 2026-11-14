//! Applies one op with the caller's values: the repeat check, the ref name check, the intent, one ref
//! transaction, the closing record; the values are written in the intent and the closing record.
//! No unit test: it reads and appends the log file and runs git (S-5). find_done_in_tests,
//! check_ref_moves_tests, ref_format_verdict_tests, ref_updates_tests and close_after_update_tests cover its
//! decisions. Nagatha's integration group should cover, in a throwaway repository: a fresh op moves every ref
//! and leaves an intent then a done record, both carrying the values, which read_valued_log returns in order;
//! a second apply with the same key returns Repeated with the first done record and that record's values (not
//! the second call's), moves nothing and appends nothing; a move with an unusable ref name (empty, holding a
//! NUL, not a full refs/ name, or refused by git check-ref-format) is refused with OplogError::BadMove before
//! anything is written; refs/heads/a..b refused with no intent written; a stale expected value moves nothing
//! and leaves an intent then a failed record naming the ref and its actual value (absent: null), both carrying
//! the values; an op with no moves closes done; an all-zero new deletes the ref; a git failure in the
//! transaction returns OplogError::Git with the intent left open; an error reading the refused ref after a
//! stale refusal returns that error with the intent open; a kill after Stale, then recover, closes an op that
//! changes a ref as failed recording each ref's actual value, and an op that changes no ref as done; a
//! reference-transaction hook sees MORTAR_OP=<op_id> on the transaction, while the name checks and the stale
//! read do not need it.

use std::path::Path;

use gitio::{Git, UpdateRefsOutcome};

use super::{check_ref_names, close_after_update, find_done_in, op_git, ref_updates, ApplyOutcome, ValuedOutcome};
use crate::store::{
    append_valued_record, intent_record, read_ref_value, read_valued_log, unix_now, Op, OplogError,
};

/// Runs `op` against the log at `<common>/mortar/oplog.jsonl` and the refs of `git`, writing `values` (the
/// caller's name/value pairs, in the caller's order) in the intent and the closing record.
///
/// The caller runs recover first, as every command does at its start. Without it, a repeat after a kill
/// between the ref transaction and the done append would find no done record, run the transaction again
/// against refs that already moved, and log a stale failure for an op that moved.
///
/// A repeat of a done key returns Repeated, with the done record's own values, before the ref names are
/// checked. A move whose ref name is empty, holds a NUL, is not a full refs/ name, or that git check-ref-format
/// refuses returns BadMove before the intent is written, so nothing is left open; a git failure while checking
/// returns OplogError::Git, also with nothing written.
///
/// The transaction runs with MORTAR_OP=<op.op_id> (op_git) so the guard hook can match it to the open intent;
/// the name check and the read after a stale refusal run with the plain `git`.
///
/// The transaction runs even with no moves: git accepts an empty one. A git error in the transaction returns
/// with the intent open, and recover settles it from the refs: done if every ref the op changes is at new,
/// failed if no ref the op changes is.
/// An error reading the refused ref after Stale also returns with the intent open; recover then closes it as
/// done when the op changes no ref; otherwise as failed, recording each ref's actual value, unless a ref the
/// op changes already holds its new value: then done when every ref the op changes is at new, and refused as
/// Mixed otherwise.
pub fn apply_with_values(
    git: &Git,
    common: &Path,
    op: &Op,
    values: &[(String, String)],
) -> Result<ValuedOutcome, OplogError> {
    if let Some(done) = find_done_in(&read_valued_log(common)?.records, &op.key, |v| &v.record) {
        return Ok(ValuedOutcome {
            outcome: ApplyOutcome::Repeated(done.record.clone()),
            values: done.values.clone(),
        });
    }
    check_ref_names(git, &op.moves)?;
    let intent = intent_record(op);
    append_valued_record(common, &intent, values)?;
    let outcome = op_git(git, &op.op_id)
        .update_refs(&ref_updates(&op.moves))
        .map_err(OplogError::Git)?;
    let actual = match &outcome {
        UpdateRefsOutcome::Updated => None,
        UpdateRefsOutcome::Stale { refname } => read_ref_value(git, refname)?,
    };
    let closed = close_after_update(&intent, &outcome, actual, unix_now());
    append_valued_record(common, &closed, values)?;
    let outcome = match outcome {
        UpdateRefsOutcome::Updated => ApplyOutcome::Done(closed),
        UpdateRefsOutcome::Stale { .. } => ApplyOutcome::Failed(closed),
    };
    Ok(ValuedOutcome {
        outcome,
        values: values.to_vec(),
    })
}
