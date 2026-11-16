//! The report of a landing verb (back-out, publish) that land's land, back_out or one of publish's ops refused, in
//! the cli's own words: land's, gitio's and oplog's Display texts are never printed; only an error's oids, op ids
//! and keys are.

use land::LandError;

use super::{landing_refusal, landing_state_error_report, squash_error_report};
use crate::report::{failure_report, Report};

/// - Squash: squash_error_report; State: landing_state_error_report.
/// - Git, Oplog, Trailers: failure_report (exit 5) naming what failed.
/// - NoZone: refused, the next step is to make the squash again.
/// - NoLandingState, IntegrationElsewhere: refused, owner refs/heads/<integration_branch>; the next step finds the
///   op that moved integration in the log and restores it.
/// - OpFailed, OpRepeated: refused, naming the record's op id or key; the next step is a repeat.
///
/// Every refusal is exit 1, outcome "refused", with no fields.
pub fn land_error_report(verb: &str, error: &LandError, integration_branch: &str) -> Report {
    let refused = |line: String, owner: &str, next: String| landing_refusal(verb, vec![line], owner, next);
    let integration_ref = format!("refs/heads/{integration_branch}");
    let restore = format!(
        "find the op that moved it with mortar log --ref {integration_ref}, then mortar restore <op id> \
         {integration_ref} --actor <name>"
    );
    match error {
        LandError::Squash(error) => squash_error_report(verb, error),
        LandError::State(error) => landing_state_error_report(verb, error),
        LandError::Git(_) => failure_report(verb, "git failed while reading or moving integration"),
        LandError::Oplog(_) => failure_report(verb, "the op log could not be read or written"),
        LandError::Trailers(_) => failure_report(verb, "the squash commit's Mortar trailers could not be read"),
        LandError::NoZone { commit } => refused(
            format!("squash commit {commit} carries no Mortar-Zone trailer"),
            "the squash commit",
            "make the squash again: mortar squash <zone> --actor <name>".to_string(),
        ),
        LandError::NoLandingState { origin, integration } => refused(
            format!(
                "integration is at {integration}, not at origin {origin}, and no landing state names a landing in \
                 flight"
            ),
            &integration_ref,
            restore,
        ),
        LandError::IntegrationElsewhere { expected, actual } => refused(
            format!("integration is at {actual}, at neither origin nor the landed commit {expected}"),
            &integration_ref,
            restore,
        ),
        LandError::OpFailed(record) => refused(
            format!("op {} moved nothing: a ref was not at its expected value", record.op_id),
            "the landing's refs",
            "repeat the same command; it reads the refs afresh".to_string(),
        ),
        LandError::OpRepeated(record) => refused(
            format!("op key {} is already done in the op log, so nothing moved", record.key),
            "the op log",
            "repeat the same command".to_string(),
        ),
    }
}
