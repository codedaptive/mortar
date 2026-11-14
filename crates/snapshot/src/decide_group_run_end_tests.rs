use crate::child_exit::ChildExit;
use crate::decide_group_run_end::decide_group_run_end;
use crate::group_run_end::GroupRunEnd;
use std::io;
use std::time::Duration;

fn exited(code: Option<i32>, stderr: &[u8]) -> io::Result<ChildExit> {
    Ok(ChildExit::Exited { code, stderr: stderr.to_vec() })
}

fn timed_out(secs: u64, stderr: &[u8]) -> io::Result<ChildExit> {
    Ok(ChildExit::TimedOut { limit: Duration::from_secs(secs), stderr: stderr.to_vec() })
}

fn ended(code: Option<i32>, stderr: &str) -> GroupRunEnd {
    GroupRunEnd::Exited { code, stderr: stderr.to_string() }
}

fn ended_timed_out(secs: u64, stderr: &str) -> GroupRunEnd {
    GroupRunEnd::TimedOut { limit: Duration::from_secs(secs), stderr: stderr.to_string() }
}

#[test]
fn exit_zero_keeps_its_stderr() {
    assert_eq!(decide_group_run_end(exited(Some(0), b"warning\n")), ended(Some(0), "warning\n"));
}

#[test]
fn a_non_zero_exit_keeps_its_code_and_stderr() {
    assert_eq!(decide_group_run_end(exited(Some(1), b"error: bad.rs\n")), ended(Some(1), "error: bad.rs\n"));
}

#[test]
fn killed_by_a_signal_is_exited_with_no_code() {
    assert_eq!(decide_group_run_end(exited(None, b"")), ended(None, ""));
}

#[test]
fn stderr_is_cut_to_4096_bytes() {
    assert_eq!(decide_group_run_end(exited(Some(2), &[b'x'; 5000])), ended(Some(2), &"x".repeat(4096)));
}

#[test]
fn stderr_at_4096_bytes_is_kept_whole() {
    assert_eq!(decide_group_run_end(exited(Some(2), &[b'y'; 4096])), ended(Some(2), &"y".repeat(4096)));
}

#[test]
fn a_spawn_error_is_spawn_failed_with_its_text() {
    let error = io::Error::new(io::ErrorKind::NotFound, "no such file");
    assert_eq!(decide_group_run_end(Err(error)), GroupRunEnd::SpawnFailed { error: "no such file".to_string() });
}

#[test]
fn invalid_utf8_in_stderr_is_replaced() {
    assert_eq!(decide_group_run_end(exited(Some(1), b"bad \xff")), ended(Some(1), "bad \u{FFFD}"));
}

#[test]
fn a_timed_out_run_is_timed_out_with_its_limit_and_stderr() {
    assert_eq!(
        decide_group_run_end(timed_out(30, b"formatting src/a.rs\n")),
        ended_timed_out(30, "formatting src/a.rs\n")
    );
}

#[test]
fn a_timed_out_run_with_no_stderr_is_timed_out_with_empty_stderr() {
    assert_eq!(decide_group_run_end(timed_out(2, b"")), ended_timed_out(2, ""));
}

#[test]
fn a_timed_out_runs_stderr_is_cut_to_4096_bytes() {
    assert_eq!(decide_group_run_end(timed_out(1, &[b'z'; 5000])), ended_timed_out(1, &"z".repeat(4096)));
}

#[test]
fn invalid_utf8_in_a_timed_out_runs_stderr_is_replaced() {
    assert_eq!(decide_group_run_end(timed_out(30, b"bad \xff")), ended_timed_out(30, "bad \u{FFFD}"));
}
