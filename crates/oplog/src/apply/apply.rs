//! Applies one op: the repeat check, the ref name check, the intent, one ref transaction, the closing record.
//! No unit test: it reads and appends the log file and runs git (S-5). find_done_record_tests,
//! check_ref_moves_tests, ref_updates_tests and close_after_update_tests cover its decisions. Nagatha's
//! integration group should cover, in a throwaway repository: a fresh op moves every ref and leaves an intent
//! then a done record; a second apply with the same key returns Repeated with the first done record, moves
//! nothing and appends nothing; a move with an unusable ref name (empty, or holding a NUL) is refused with
//! OplogError::BadMove before anything is written; a stale expected value moves nothing and leaves an intent
//! then a failed record naming the ref and its actual value (absent: null); an op with no moves closes done;
//! an all-zero new deletes the ref; a git failure in the transaction returns OplogError::Git with the intent
//! left open; an error reading the refused ref after a stale refusal returns that error with the intent open.

use std::path::Path;

use gitio::{Git, UpdateRefsOutcome};

use super::{check_ref_moves, close_after_update, find_done_record, ref_updates, ApplyOutcome};
use crate::store::{append_op_record, intent_record, read_op_log, read_ref_value, unix_now, Op, OplogError};

/// Runs `op` against the log at `<common>/mortar/oplog.jsonl` and the refs of `git`.
///
/// The caller runs recover first, as every command does at its start. Without it, a repeat after a kill
/// between the ref transaction and the done append would find no done record, run the transaction again
/// against refs that already moved, and log a stale failure for an op that moved.
///
/// A repeat of a done key returns Repeated before the ref names are checked. A move whose ref name is empty
/// or holds a NUL returns BadMove before the intent is written, so nothing is left open.
///
/// The transaction runs even with no moves: git accepts an empty one. A git error in the transaction returns
/// with the intent open, and recover settles it from the refs: done if they all moved, failed if none did.
/// An error reading the refused ref after Stale also returns with the intent open; recover then refuses it as
/// Mixed, naming the op and its refs, since that ref is not at its old value; if another actor has already
/// set it to its new value and every other ref is at new too, recover closes the op as done instead.
pub fn apply(git: &Git, common: &Path, op: &Op) -> Result<ApplyOutcome, OplogError> {
    if let Some(done) = find_done_record(&read_op_log(common)?.records, &op.key) {
        return Ok(ApplyOutcome::Repeated(done.clone()));
    }
    check_ref_moves(&op.moves)?;
    let intent = intent_record(op);
    append_op_record(common, &intent)?;
    let outcome = git
        .update_refs(&ref_updates(&op.moves))
        .map_err(OplogError::Git)?;
    let actual = match &outcome {
        UpdateRefsOutcome::Updated => None,
        UpdateRefsOutcome::Stale { refname } => read_ref_value(git, refname)?,
    };
    let closed = close_after_update(&intent, &outcome, actual, unix_now());
    append_op_record(common, &closed)?;
    Ok(match outcome {
        UpdateRefsOutcome::Updated => ApplyOutcome::Done(closed),
        UpdateRefsOutcome::Stale { .. } => ApplyOutcome::Failed(closed),
    })
}
