//! restore_moves: the moves an `mr restore` makes, read before it runs so the folders on those refs can be
//! checked first.
//! No unit test: it reads the op log and runs git; its decisions are oplog's (find_done_record,
//! check_restore_ref_names, plan_restore), covered there, and restore_refusal_tests cover what restore decides from
//! the moves. The validation tests are to cover, with the mortar binary as a child process in a throwaway
//! repository (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - a restore that would delete a folder's ref refuses exit 1.
//!   The validation test crates/cli/tests/validation/run_restore_repeat_tests.rs covers a kill between oplog::restore and the folder
//!   move finished by a plain repeat (Repeated, then the move). The validation test crates/cli/tests/validation/run_restore_tests.rs
//!   covers a ref name git refuses coming back as BadMove, exit 1, with no ref written and no record added.

use std::path::Path;

use gitio::Git;
use oplog::{
    check_restore_ref_names, find_done_record, plan_restore, read_op_log, read_ref_value, OplogError, RefMove,
};

/// The moves and whether they are a repeat's. When the op log has a done record with `key`, its moves and true: the
/// restore already ran, and oplog::restore will answer Repeated with that record, so a repeat checks and moves the
/// folders by what the restore did. Else the names are checked as oplog::restore checks them, with
/// oplog::check_restore_ref_names before any ref is read (a name git refuses is its BadMove), then the moves oplog's
/// plan_restore makes from the log and each ref's current value, as oplog::restore plans them under the same lock,
/// and false: UnknownOp, ValueUnknown and the op log's own refusals come back as they are.
pub fn restore_moves(
    git: &Git,
    common: &Path,
    key: &str,
    op_id: &str,
    refs: &[String],
) -> Result<(Vec<RefMove>, bool), OplogError> {
    let records = read_op_log(common)?.records;
    if let Some(done) = find_done_record(&records, key) {
        return Ok((done.moves.clone(), true));
    }
    check_restore_ref_names(git, refs)?;
    let current = refs
        .iter()
        .map(|refname| Ok((refname.clone(), read_ref_value(git, refname)?)))
        .collect::<Result<Vec<_>, OplogError>>()?;
    Ok((plan_restore(&records, op_id, &current)?, false))
}
