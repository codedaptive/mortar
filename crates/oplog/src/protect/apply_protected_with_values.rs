//! Applies one op with the caller's values as apply_with_values does, after refusing any move of a protected ref
//! that is not a fast-forward or a creation.
//! No unit test: it reads the log file and runs git; find_done_in_tests, check_ref_moves_tests and
//! protected_move_verdict_tests cover its decisions. Covered by the crate's integration tests
//! (tests/oplog/protected_values.rs), in a throwaway repository: a done key returns Repeated with the first done
//! record and its values (not the second call's) and writes nothing, even when the protected move would now be
//! refused; a name that is not a full refs/ name returns BadMove, and a rewind, a sideways move or a delete of a
//! protected ref returns Protected, each with nothing written to the log and no ref moved; a
//! fast-forward of a protected ref applies Done, with the intent and the done record both carrying the values.

use std::path::Path;

use gitio::Git;

use super::{check_protected_moves, ProtectedRef};
use crate::apply::{check_ref_moves, find_done_in, ApplyOutcome, ValuedOutcome};
use crate::store::{read_valued_log, Op, OplogError};

/// Runs `op` as apply_with_values runs it, writing `values` in the intent and the closing record, with the moves
/// of the refs in `protected` held to fast-forward or creation.
///
/// The caller runs recover first, as for apply_with_values. A repeat of a done key returns Repeated, with the done
/// record's own values, before any protected ref is read. Then check_ref_moves refuses an empty, NUL-holding or
/// non-refs/ name as BadMove before any ref is read, and a refused protected move returns Protected; both come
/// before the intent, so nothing is written. The rest runs through apply_with_values, which carries MORTAR_OP on
/// the ref transaction (op_git).
pub fn apply_protected_with_values(
    git: &Git,
    common: &Path,
    op: &Op,
    protected: &[ProtectedRef],
    values: &[(String, String)],
) -> Result<ValuedOutcome, OplogError> {
    if let Some(done) = find_done_in(&read_valued_log(common)?.records, &op.key, |v| &v.record) {
        return Ok(ValuedOutcome {
            outcome: ApplyOutcome::Repeated(done.record.clone()),
            values: done.values.clone(),
        });
    }
    check_ref_moves(&op.moves)?;
    check_protected_moves(git, &op.moves, protected)?;
    crate::apply::apply_with_values(git, common, op, values)
}
