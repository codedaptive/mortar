use crate::decide_formatter_result::decide_formatter_result;
use crate::formatter_result::FormatterResult;
use crate::group_run_end::GroupRunEnd;
use std::time::Duration;

fn exited(code: Option<i32>, stderr: &str) -> GroupRunEnd {
    GroupRunEnd::Exited { code, stderr: stderr.to_string() }
}

fn timed_out(secs: u64, stderr: &str) -> GroupRunEnd {
    GroupRunEnd::TimedOut { limit: Duration::from_secs(secs), stderr: stderr.to_string() }
}

fn failed(code: Option<i32>, stderr: &str) -> FormatterResult {
    FormatterResult::Failed { code, stderr: stderr.to_string() }
}

fn listed_timed_out(secs: u64, stderr: &str) -> FormatterResult {
    FormatterResult::TimedOut { limit: Duration::from_secs(secs), stderr: stderr.to_string() }
}

#[test]
fn exit_zero_is_formatted_and_drops_stderr() {
    assert_eq!(decide_formatter_result(exited(Some(0), "warning\n")), FormatterResult::Formatted);
}

#[test]
fn a_non_zero_exit_is_failed_with_its_code_and_stderr() {
    assert_eq!(decide_formatter_result(exited(Some(1), "error: bad.rs\n")), failed(Some(1), "error: bad.rs\n"));
}

#[test]
fn killed_by_a_signal_is_failed_with_no_code() {
    assert_eq!(decide_formatter_result(exited(None, "")), failed(None, ""));
}

#[test]
fn a_spawn_error_is_spawn_failed_with_its_text() {
    let end = GroupRunEnd::SpawnFailed { error: "no such file".to_string() };
    assert_eq!(decide_formatter_result(end), FormatterResult::SpawnFailed { error: "no such file".to_string() });
}

#[test]
fn a_timed_out_run_is_timed_out_with_its_limit_and_stderr() {
    assert_eq!(
        decide_formatter_result(timed_out(30, "formatting src/a.rs\n")),
        listed_timed_out(30, "formatting src/a.rs\n")
    );
}

#[test]
fn a_timed_out_run_with_no_stderr_is_timed_out_with_empty_stderr() {
    assert_eq!(decide_formatter_result(timed_out(2, "")), listed_timed_out(2, ""));
}
