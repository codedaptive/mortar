//! Applies one op as apply does, after refusing any move of a protected ref that is not a fast-forward or a
//! creation.
//! No unit test: it reads the log file and runs git; find_done_record_tests, check_ref_moves_tests and
//! protected_move_verdict_tests cover its decisions. Covered by the crate's integration tests
//! (tests/oplog/protection.rs), in a throwaway repository: a done key returns Repeated even when a protected ref
//! would now be refused; a rewind, a sideways move or a delete of a protected ref returns Protected with nothing
//! appended to the log; a fast-forward or a creation of a protected ref applies as apply does; unprotected refs
//! move as apply moves them.

use std::path::Path;

use gitio::Git;

use super::{check_protected_moves, ProtectedRef};
use crate::apply::{check_ref_moves, find_done_record, ApplyOutcome};
use crate::store::{read_op_log, Op, OplogError};

/// Runs `op` as apply runs it, with the moves of the refs in `protected` held to fast-forward or creation.
///
/// The caller runs recover first, as for apply. A repeat of a done key returns Repeated before any protected
/// ref is read. Then check_ref_moves refuses an empty, NUL-holding or non-refs/ name as BadMove before any ref
/// is read, and a refused protected move returns Protected; both come before the intent, so nothing is
/// written. The ref transaction then runs through apply, which carries MORTAR_OP on it (op_git); apply keeps
/// its own behaviour for callers that protect nothing.
pub fn apply_protected(
    git: &Git,
    common: &Path,
    op: &Op,
    protected: &[ProtectedRef],
) -> Result<ApplyOutcome, OplogError> {
    if let Some(done) = find_done_record(&read_op_log(common)?.records, &op.key) {
        return Ok(ApplyOutcome::Repeated(done.clone()));
    }
    check_ref_moves(&op.moves)?;
    check_protected_moves(git, &op.moves, protected)?;
    crate::apply::apply(git, common, op)
}
