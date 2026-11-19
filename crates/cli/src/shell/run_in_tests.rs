//! Pure tests of run_in's git floor and folder-read ordering.

use std::cell::Cell;
use std::path::Path;

use gitio::Version;

use super::{git_floor_report, run_in, ShellIo};
use crate::context::{context_report, ContextError};
use crate::report::{ExitCode, Report};

/// A floor refusal must precede the folder read, even if its report would still win afterward.
#[test]
fn below_floor_refuses_before_reading_folder() {
    let version_calls = Cell::new(0);
    let folder_calls = Cell::new(0);
    let cwd = Path::new("/virtual/mortar-folder");
    let read_version = || {
        version_calls.set(version_calls.get() + 1);
        Version { major: 2, minor: 45, patch: 2 }.require_floor()
    };
    let read_folder = |path: &Path| {
        folder_calls.set(folder_calls.get() + 1);
        Err(ContextError::NotAFolder { toplevel: path.to_path_buf() })
    };
    let io = ShellIo { read_version: &read_version, read_folder: &read_folder };
    let error = Version { major: 2, minor: 45, patch: 2 }.require_floor().unwrap_err();

    let (report, json) = run_in(&["status".to_string()], cwd, &io);

    assert_eq!(report, git_floor_report("status", &error));
    assert!(!json);
    assert_eq!(version_calls.get(), 1);
    assert_eq!(folder_calls.get(), 0);
}

#[test]
fn floor_version_proceeds_to_folder_read() {
    let version_calls = Cell::new(0);
    let folder_calls = Cell::new(0);
    let cwd = Path::new("/virtual/mortar-folder");
    let read_version = || {
        version_calls.set(version_calls.get() + 1);
        Version { major: 2, minor: 46, patch: 0 }.require_floor()
    };
    let read_folder = |path: &Path| {
        folder_calls.set(folder_calls.get() + 1);
        Err(ContextError::NotAFolder { toplevel: path.to_path_buf() })
    };
    let io = ShellIo { read_version: &read_version, read_folder: &read_folder };
    let error = ContextError::NotAFolder { toplevel: cwd.to_path_buf() };

    let (report, json) = run_in(&["status".to_string()], cwd, &io);

    assert_eq!(report, context_report("status", &error));
    assert!(!json);
    assert_eq!(version_calls.get(), 1);
    assert_eq!(folder_calls.get(), 1);
}

#[test]
fn unreadable_version_refuses_before_reading_folder() {
    let version_calls = Cell::new(0);
    let folder_calls = Cell::new(0);
    let cwd = Path::new("/virtual/mortar-folder");
    let read_version = || {
        version_calls.set(version_calls.get() + 1);
        Version::parse(b"not git\n")
    };
    let read_folder = |path: &Path| {
        folder_calls.set(folder_calls.get() + 1);
        Err(ContextError::NotAFolder { toplevel: path.to_path_buf() })
    };
    let io = ShellIo { read_version: &read_version, read_folder: &read_folder };
    let error = Version::parse(b"not git\n").unwrap_err();

    let (report, json) = run_in(&["status".to_string()], cwd, &io);

    assert_eq!(report, git_floor_report("status", &error));
    assert!(!json);
    assert_eq!(version_calls.get(), 1);
    assert_eq!(folder_calls.get(), 0);
}

#[test]
fn help_skips_both_reads_and_reports_no_runner() {
    let version_calls = Cell::new(0);
    let folder_calls = Cell::new(0);
    let cwd = Path::new("/virtual/mortar-folder");
    let read_version = || {
        version_calls.set(version_calls.get() + 1);
        Version { major: 2, minor: 45, patch: 2 }.require_floor()
    };
    let read_folder = |path: &Path| {
        folder_calls.set(folder_calls.get() + 1);
        Err(ContextError::NotAFolder { toplevel: path.to_path_buf() })
    };
    let io = ShellIo { read_version: &read_version, read_folder: &read_folder };

    let (report, json) = run_in(&["help".to_string()], cwd, &io);

    assert_eq!(
        report,
        Report {
            verb: "help".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec!["the verb help is in the verb table but has no runner".to_string()],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
    assert!(!json);
    assert_eq!(version_calls.get(), 0);
    assert_eq!(folder_calls.get(), 0);
}
