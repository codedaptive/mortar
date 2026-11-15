//! Sets refs to their values as of an op, as one new op of kind restore through apply_protected.
//! No unit test: it reads the log file and runs git (S-5); find_done_record_tests, check_restore_ref_names_tests,
//! name_moves_tests, check_ref_moves_tests, ref_format_verdict_tests, plan_restore_tests, value_as_of_op_tests and
//! caller_op_tests cover its decisions, and tests/oplog/restore_ref_names.rs covers BadMove for a refused name with
//! the log and refs unchanged. Nagatha's integration group should cover, in a throwaway repository: restore to an
//! older value, a delete for a ref absent then, a create for a ref absent now; UnknownOp for an op id the log does
//! not hold; ValueUnknown for a ref no done op moved; a repeat with the same key returns Repeated and moves once; a
//! protected ref refused as Protected on a rewind; the new op logged with kind restore.

use std::path::Path;

use gitio::Git;

use super::{caller_op, check_restore_ref_names, plan_restore, OpCaller};
use crate::apply::{find_done_record, new_op_id, ApplyOutcome};
use crate::protect::{apply_protected, ProtectedRef};
use crate::store::{read_op_log, read_ref_value, unix_now, OplogError};

/// Sets each ref in `refs` to what it held as of `op_id`, with the refs in `protected` held to fast-forward or
/// creation; refs already there are not moved.
///
/// The caller runs recover first and holds its zone's join lock or the landing lock around the call (S-16), as
/// for apply. A repeat of the caller's done key returns Repeated before anything is planned, so a repeat after
/// the refs moved again still moves them once. Then, before any ref is read, an empty, NUL-holding, non-refs/ or
/// git check-ref-format-refused name in `refs` returns BadMove, as apply does (S-14).
pub fn restore(
    git: &Git,
    common: &Path,
    op_id: &str,
    refs: &[String],
    caller: &OpCaller,
    protected: &[ProtectedRef],
) -> Result<ApplyOutcome, OplogError> {
    let records = read_op_log(common)?.records;
    if let Some(done) = find_done_record(&records, &caller.key) {
        return Ok(ApplyOutcome::Repeated(done.clone()));
    }
    check_restore_ref_names(git, refs)?;
    let current = refs
        .iter()
        .map(|r| Ok((r.clone(), read_ref_value(git, r)?)))
        .collect::<Result<Vec<_>, OplogError>>()?;
    let moves = plan_restore(&records, op_id, &current)?;
    let op = caller_op(caller, "restore", new_op_id(), unix_now(), moves);
    apply_protected(git, common, &op, protected)
}
