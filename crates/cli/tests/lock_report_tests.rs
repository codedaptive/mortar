//! lock_report: a file system error and an unknown host are failures. A held lock, a takeover, contention and a bad
//! zone are the ZUTs in src/lock/lock_report_tests.rs.
//! ZIT (S-79 as replaced: the line is the file): it calls report::failure_report for real.

use std::path::PathBuf;

use oplog::LockError;

use cli::{lock_report, ExitCode};

#[test]
fn a_file_system_error_is_a_failure() {
    let error = LockError::Io {
        path: PathBuf::from("/r/.git/mortar/locks"),
        source: std::io::Error::other("disk full"),
        removed_guards: Vec::new(),
    };
    let report = lock_report("join", &error);
    assert_eq!(report.exit, ExitCode::Failure);
    assert_eq!(report.outcome, "failed");
}

#[test]
fn an_unknown_host_is_a_failure() {
    assert_eq!(lock_report("join", &LockError::HostUnknown("no hostname".to_string())).exit as i32, 5);
}
