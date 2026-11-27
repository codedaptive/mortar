//! read_batch: the landing batch's members in landing order, in the cli's words, read by land and publish.
//! No sibling test: it only reads the landing state file (land's read_landing_batch); landing_state_error_report_tests
//! covers its error's text. The integration tests crates/cli/tests/land_runs/land_flow/batch.rs (the batch's members
//! named by land and the zones publish names) and run_publish_tests.rs reach it through the binary.

use gitio::Git;
use land::BatchMember;

use super::landing_state_error_report;
use crate::report::Report;

/// read_landing_batch's members; no landing state is an empty list; an Err is landing_state_error_report(verb, error).
pub fn read_batch(git: &Git, verb: &str) -> Result<Vec<BatchMember>, Report> {
    land::read_landing_batch(git)
        .map(Option::unwrap_or_default)
        .map_err(|error| landing_state_error_report(verb, &error))
}
