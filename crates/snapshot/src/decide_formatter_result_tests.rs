use crate::child_exit::ChildExit;
use crate::decide_formatter_result::decide_formatter_result;
use crate::formatter_result::FormatterResult;
use std::io;
use std::time::Duration;

fn exited(code: Option<i32>, stderr: &[u8]) -> io::Result<ChildExit> {
    Ok(ChildExit::Exited { code, stderr: stderr.to_vec() })
}

fn timed_out(secs: u64, stderr: &[u8]) -> io::Result<ChildExit> {
    Ok(ChildExit::TimedOut { limit: Duration::from_secs(secs), stderr: stderr.to_vec() })
}

fn failed(code: Option<i32>, stderr: &str) -> FormatterResult {
    FormatterResult::Failed { code, stderr: stderr.to_string() }
}

fn listed_timed_out(secs: u64, stderr: &str) -> FormatterResult {
    FormatterResult::TimedOut { limit: Duration::from_secs(secs), stderr: stderr.to_string() }
}

#[test]
fn exit_zero_is_formatted_and_drops_stderr() {
    assert_eq!(decide_formatter_result(exited(Some(0), b"warning\n")), FormatterResult::Formatted);
}

#[test]
fn a_non_zero_exit_is_failed_with_its_code_and_stderr() {
    assert_eq!(decide_formatter_result(exited(Some(1), b"error: bad.rs\n")), failed(Some(1), "error: bad.rs\n"));
}

#[test]
fn killed_by_a_signal_is_failed_with_no_code() {
    assert_eq!(decide_formatter_result(exited(None, b"")), failed(None, ""));
}

#[test]
fn stderr_is_cut_to_4096_bytes() {
    assert_eq!(decide_formatter_result(exited(Some(2), &[b'x'; 5000])), failed(Some(2), &"x".repeat(4096)));
}

#[test]
fn stderr_at_4096_bytes_is_kept_whole() {
    assert_eq!(decide_formatter_result(exited(Some(2), &[b'y'; 4096])), failed(Some(2), &"y".repeat(4096)));
}

#[test]
fn a_spawn_error_is_spawn_failed_with_its_text() {
    let error = io::Error::new(io::ErrorKind::NotFound, "no such file");
    assert_eq!(decide_formatter_result(Err(error)), FormatterResult::SpawnFailed { error: "no such file".to_string() });
}

#[test]
fn invalid_utf8_in_stderr_is_replaced() {
    assert_eq!(decide_formatter_result(exited(Some(1), b"bad \xff")), failed(Some(1), "bad \u{FFFD}"));
}

#[test]
fn a_timed_out_run_is_timed_out_with_its_limit_and_stderr() {
    assert_eq!(
        decide_formatter_result(timed_out(30, b"formatting src/a.rs\n")),
        listed_timed_out(30, "formatting src/a.rs\n")
    );
}

#[test]
fn a_timed_out_run_with_no_stderr_is_timed_out_with_empty_stderr() {
    assert_eq!(decide_formatter_result(timed_out(2, b"")), listed_timed_out(2, ""));
}

#[test]
fn a_timed_out_runs_stderr_is_cut_to_4096_bytes() {
    assert_eq!(decide_formatter_result(timed_out(1, &[b'z'; 5000])), listed_timed_out(1, &"z".repeat(4096)));
}

#[test]
fn invalid_utf8_in_a_timed_out_runs_stderr_is_replaced() {
    assert_eq!(decide_formatter_result(timed_out(30, b"bad \xff")), listed_timed_out(30, "bad \u{FFFD}"));
}
