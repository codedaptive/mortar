//! Probe texts: a pass, a fail with an exit code, a signal kill, and timeouts in whole seconds and in milliseconds.

use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;

use super::{Probe, ProbeOutcome};

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn probe(outcome: ProbeOutcome) -> Probe { Probe { commit: a(), outcome, folder: PathBuf::from("unused-probe-folder") } }

#[test]
fn a_pass_reads_passed() {
    assert_eq!(probe(ProbeOutcome::Passed).to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: passed");
}

#[test]
fn a_fail_names_its_exit_code() {
    assert_eq!(probe(ProbeOutcome::Failed { code: Some(1) }).to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed with exit code 1");
}

#[test]
fn a_signal_kill_says_so() {
    assert_eq!(probe(ProbeOutcome::Failed { code: None }).to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, killed by a signal");
}

#[test]
fn a_timeout_says_timed_out_with_whole_seconds() {
    assert_eq!(probe(ProbeOutcome::TimedOut { limit: Duration::from_secs(60) }).to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, timed out after 60s");
}

#[test]
fn a_timeout_of_part_seconds_reads_in_milliseconds() {
    assert_eq!(probe(ProbeOutcome::TimedOut { limit: Duration::from_millis(1500) }).to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: failed, timed out after 1500ms");
}
