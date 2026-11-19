//! Report texts for the NotFound answers with no probes. The culprit lines and the probe lines name a commit or a
//! change ID (gitio's and oplog's Display), so they are repo integration tests (RITs):
//! tests/land_rit/owed_displays.rs covers them.

use super::{BisectAnswer, BisectNotFound, BisectReport};

fn report(answer: BisectAnswer) -> BisectReport {
    BisectReport { answer, probes: Vec::new() }
}

#[test]
fn failed_passes_says_the_failed_commit_passes() {
    assert_eq!(
        report(BisectAnswer::NotFound(BisectNotFound::FailedPasses)).to_string(),
        "no culprit: the failed commit passes the command"
    );
}

#[test]
fn last_pass_fails_says_the_last_passing_commit_fails() {
    assert_eq!(
        report(BisectAnswer::NotFound(BisectNotFound::LastPassFails)).to_string(),
        "no culprit: the last passing commit fails the command"
    );
}
