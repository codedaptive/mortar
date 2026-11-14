//! failure_report: exit 5, what failed then that nothing half-done is left, and a repeat as next.

use super::{failure_report, ExitCode, Report};

#[test]
fn a_failure_says_nothing_is_half_done_and_to_repeat() {
    assert_eq!(
        failure_report("init", "cannot write /r/.git/mortar/config.json: disk full"),
        Report {
            verb: "init".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "cannot write /r/.git/mortar/config.json: disk full".to_string(),
                "nothing half-done is left".to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn the_failure_code_is_5() {
    assert_eq!(failure_report("", "git did not start").exit as i32, 5);
}
