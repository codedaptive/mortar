//! lock_report: a held lock or a takeover is busy naming the holder; contention is busy; a bad zone is refused;
//! the rest are failures.
//! ZIT (S-79 as replaced: the line is the file): it calls report::failure_report for real.

use std::path::PathBuf;

use oplog::{LockError, LockRecord};

use cli::{lock_report, ExitCode, OutValue, Report};

fn holder() -> LockRecord {
    LockRecord { pid: 4242, host: "build-7".to_string(), key: "join-a1b2".to_string(), since: 1_763_100_000 }
}

#[test]
fn a_held_lock_is_busy_naming_the_holder_and_since() {
    let error = LockError::Held {
        path: PathBuf::from("/r/.git/mortar/locks/join-orc.lock"),
        holder: holder(),
        removed_guards: Vec::new(),
    };
    assert_eq!(
        lock_report("join", &error),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Busy,
            outcome: "busy".to_string(),
            lines: vec![
                "the lock /r/.git/mortar/locks/join-orc.lock is held by pid 4242 on build-7 for op join-a1b2 \
                 since 1763100000 (unix seconds); nothing ran"
                    .to_string()
            ],
            owner: Some("pid 4242 on build-7".to_string()),
            next: "repeat the same command later".to_string(),
            fields: vec![
                ("holder_pid".to_string(), OutValue::Number(4242)),
                ("holder_host".to_string(), OutValue::Text("build-7".to_string())),
                ("holder_key".to_string(), OutValue::Text("join-a1b2".to_string())),
                ("since".to_string(), OutValue::Number(1_763_100_000)),
            ],
        }
    );
}

#[test]
fn a_takeover_in_progress_is_busy_naming_the_guard_holder() {
    let error = LockError::TakeoverInProgress {
        path: PathBuf::from("/r/.git/mortar/locks/land.lock.takeover"),
        holder: holder(),
        removed_guards: Vec::new(),
    };
    let report = lock_report("land", &error);
    assert_eq!(report.exit as i32, 3);
    assert_eq!(
        report.lines,
        vec!["a takeover of the lock is in progress: /r/.git/mortar/locks/land.lock.takeover is held by pid 4242 \
              on build-7 for op join-a1b2 since 1763100000 (unix seconds); nothing ran"
            .to_string()]
    );
    assert_eq!(report.owner.as_deref(), Some("pid 4242 on build-7"));
    assert_eq!(report.fields[0], ("holder_pid".to_string(), OutValue::Number(4242)));
}

#[test]
fn contention_is_busy_without_a_holder() {
    let error = LockError::Contended { path: PathBuf::from("/r/.git/mortar/locks/land.lock"), removed_guards: Vec::new() };
    let report = lock_report("land", &error);
    assert_eq!(report.exit, ExitCode::Busy);
    assert_eq!(report.outcome, "busy");
    assert_eq!(report.owner, None);
    assert_eq!(report.fields, Vec::new());
    assert_eq!(report.next, "repeat the same command later");
}

#[test]
fn a_bad_zone_name_is_refused() {
    let report = lock_report("join", &LockError::BadName("../x".to_string()));
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.lines, vec!["the zone \"../x\" cannot name a lock file; nothing ran".to_string()]);
}

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
