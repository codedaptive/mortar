//! The report of a landing verb that land's withdraw refused, in the cli's own words: land's, gitio's and oplog's
//! Display texts are never printed; only the error's zone, commits, change ID, op id and key are.

use land::{SquashError, WithdrawError};

use super::{landing_refusal, landing_state_error_report, squash_error_report};
use crate::report::{failure_report, Report};

/// - Squash: as squash_error_report words it (from reading the origin tip or the squash ref).
/// - Git, Oplog: failure_report (exit 5) naming what failed. State: as landing_state_error_report words it.
/// - Published: refused, the squash is in origin; nothing was moved; the next step is new work and a new squash.
/// - Landing: refused, the squash is the landing in flight; nothing was moved; the next step is back-out, then
///   withdraw again.
/// - OpFailed, OpRepeated: as squash_error_report words them.
///
/// Every refusal is exit 1, outcome "refused", with no fields.
pub fn withdraw_error_report(verb: &str, error: &WithdrawError) -> Report {
    let nothing_moved = |line: String| vec![line, "nothing was moved".to_string()];
    match error {
        WithdrawError::Squash(error) => squash_error_report(verb, error),
        WithdrawError::Git(_) => failure_report(
            verb,
            "git failed while reading the repository's common dir or building the op that deletes the squash ref",
        ),
        WithdrawError::Oplog(_) => failure_report(verb, "the op log could not be read or written"),
        WithdrawError::State(error) => landing_state_error_report(verb, error),
        WithdrawError::Published { zone, commit, origin } => landing_refusal(
            verb,
            nothing_moved(format!(
                "squash {commit} of zone {zone} is already in origin (at {origin}): it was published and cannot be \
                 withdrawn"
            )),
            "the squash ref",
            format!("commit new work on zone {zone}, then mr squash {zone} --actor <name> starts a new change"),
        ),
        WithdrawError::Landing { zone, commit, change_id } => landing_refusal(
            verb,
            nothing_moved(format!(
                "squash {commit} of zone {zone} is the landing in flight (change {change_id}), so it cannot be \
                 withdrawn"
            )),
            "the landing in flight",
            format!("mr back-out --actor <name>, then mr withdraw {zone} --actor <name>"),
        ),
        WithdrawError::OpFailed(record) => squash_error_report(verb, &SquashError::OpFailed(record.clone())),
        WithdrawError::OpRepeated(record) => squash_error_report(verb, &SquashError::OpRepeated(record.clone())),
    }
}
