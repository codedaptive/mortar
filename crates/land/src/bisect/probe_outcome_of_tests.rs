//! probe_outcome_of: exit 0 passes, other codes and signals fail, a timeout keeps its limit, a spawn failure errs.

use std::time::Duration;

use snapshot::GroupRunEnd;

use super::{probe_outcome_of, ProbeOutcome};

#[test]
fn exit_zero_passes() {
    let end = GroupRunEnd::Exited { code: Some(0), stderr: "noise".to_string() };
    assert_eq!(probe_outcome_of(end), Ok(ProbeOutcome::Passed));
}

#[test]
fn a_nonzero_exit_fails_with_its_code() {
    let end = GroupRunEnd::Exited { code: Some(1), stderr: String::new() };
    assert_eq!(probe_outcome_of(end), Ok(ProbeOutcome::Failed { code: Some(1) }));
}

#[test]
fn a_negative_exit_code_fails_with_its_code() {
    let end = GroupRunEnd::Exited { code: Some(-3), stderr: String::new() };
    assert_eq!(probe_outcome_of(end), Ok(ProbeOutcome::Failed { code: Some(-3) }));
}

#[test]
fn a_signal_kill_fails_with_no_code() {
    let end = GroupRunEnd::Exited { code: None, stderr: String::new() };
    assert_eq!(probe_outcome_of(end), Ok(ProbeOutcome::Failed { code: None }));
}

#[test]
fn a_timeout_is_timed_out_with_its_limit() {
    let end = GroupRunEnd::TimedOut { limit: Duration::from_secs(60), stderr: "partial".to_string() };
    assert_eq!(probe_outcome_of(end), Ok(ProbeOutcome::TimedOut { limit: Duration::from_secs(60) }));
}

#[test]
fn a_spawn_failure_is_its_error_text() {
    let end = GroupRunEnd::SpawnFailed { error: "No such file or directory (os error 2)".to_string() };
    assert_eq!(probe_outcome_of(end), Err("No such file or directory (os error 2)".to_string()));
}
