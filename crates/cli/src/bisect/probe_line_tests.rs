//! probe_line: one line per probe outcome, naming the commit probed and how the test command ended there.

use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;
use land::{Probe, ProbeOutcome};

use super::probe_line;

const COMMIT: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";

fn probe(outcome: ProbeOutcome) -> Probe {
    Probe { commit: Oid::parse(COMMIT).unwrap(), outcome, folder: PathBuf::from("/tmp/mortar-probe-1") }
}

#[test]
fn passed_says_passed() {
    assert_eq!(probe_line(&probe(ProbeOutcome::Passed)), "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: passed");
}

#[test]
fn failed_with_a_code_names_the_exit_code() {
    assert_eq!(
        probe_line(&probe(ProbeOutcome::Failed { code: Some(1) })),
        "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed with exit 1"
    );
    assert_eq!(
        probe_line(&probe(ProbeOutcome::Failed { code: Some(101) })),
        "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed with exit 101"
    );
}

#[test]
fn failed_without_a_code_says_a_signal_killed_it() {
    assert_eq!(
        probe_line(&probe(ProbeOutcome::Failed { code: None })),
        "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: failed, killed by a signal"
    );
}

#[test]
fn timed_out_names_the_limit_in_whole_seconds_and_counts_as_a_fail() {
    assert_eq!(
        probe_line(&probe(ProbeOutcome::TimedOut { limit: Duration::from_secs(30) })),
        "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: timed out after 30 s, counted as a fail"
    );
    assert_eq!(
        probe_line(&probe(ProbeOutcome::TimedOut { limit: Duration::from_millis(90_500) })),
        "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: timed out after 90 s, counted as a fail"
    );
}
