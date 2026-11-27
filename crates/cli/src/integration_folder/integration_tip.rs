//! integration_tip: the integration branch's tip, read by land and publish around land's call.
//! No sibling test: it only runs git (read_ref). The integration tests in crates/cli/tests/land_runs/land_flow/
//! (landing.rs and batch.rs for land) and crates/cli/tests/land_runs/run_publish_tests.rs reach it through the
//! binary.

use gitio::{Git, Oid};

use crate::report::{failure_report, Report};

/// refs/heads/<integration>'s tip; a ref git cannot read, or one that is missing, is `verb`'s failure_report "git
/// could not read the integration branch <integration>".
pub fn integration_tip(git: &Git, verb: &str, integration: &str) -> Result<Oid, Report> {
    match git.read_ref(&format!("refs/heads/{integration}")) {
        Ok(Some(tip)) => Ok(tip),
        _ => Err(failure_report(verb, &format!("git could not read the integration branch {integration}"))),
    }
}
