use crate::support::{assert_snapshot_written, blob, formatter_script, write, Fixture};
use snapshot::{is_saved, snapshot_formatted, Formatter, FormatterResult, DEFAULT_SIZE_CAP};
use std::{fs, path::PathBuf};
use testkit::write_git_shim;

#[test]
fn rewriting_formatter_runs_once_with_literal_argv_in_folder_and_snapshot_saves_its_output() {
    // Joins snapshot_formatted -> real child argv/cwd/file writes -> gitio tree/commit -> oplog apply.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"unformatted tracked\n");
    write(&f.repo.path, "-dash.txt", b"unformatted dash\n");
    write(&f.repo.path, "space file.txt", b"unformatted space\n");
    let before_index = fs::read(f.common.join("index")).unwrap();
    let (formatter, trace) = formatter_script(&f, b"for path in \"$@\"; do printf 'formatted\\n' >\"$path\"; done\n");
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    let formatted = saved.formatted.as_ref().unwrap();
    assert_eq!(formatted.program, formatter.program);
    assert_eq!(formatted.args, formatter.args);
    assert_eq!(formatted.paths, vec![PathBuf::from("root.txt"), PathBuf::from("-dash.txt"), PathBuf::from("space file.txt")]);
    assert_eq!(formatted.result, FormatterResult::Formatted);
    assert_eq!(fs::read(&trace).unwrap(), b"literal $(touch unexpected)\0--\0root.txt\0-dash.txt\0space file.txt\0");
    assert_eq!(fs::read(trace.with_extension("count")).unwrap(), b"run\n");
    let cwd = fs::read_to_string(trace.with_extension("cwd")).unwrap();
    assert_eq!(fs::canonicalize(cwd.trim_end()).unwrap(), fs::canonicalize(&f.repo.path).unwrap());
    for name in ["root.txt", "-dash.txt", "space file.txt"] {
        assert_eq!(blob(&f.git, &saved.tree, name), b"formatted\n");
        assert_eq!(fs::read(f.repo.path.join(name)).unwrap(), b"formatted\n");
    }
    assert!(!f.repo.path.join("unexpected").exists(), "configured argv must not be shell-evaluated");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
    let log_before = fs::read(oplog::oplog_path(&f.common)).unwrap();
    let trace_before = fs::read(&trace).unwrap();
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    write(&f.repo.path, "root.txt", b"dirty again\n");
    assert!(!is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    assert_eq!(fs::read(&trace).unwrap(), trace_before);
    assert_eq!(fs::read(trace.with_extension("count")).unwrap(), b"run\n");
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"dirty again\n");
    assert_eq!(fs::read(oplog::oplog_path(&f.common)).unwrap(), log_before);
}

#[test]
fn exit_one_is_reported_with_code_and_stderr_but_snapshot_still_records_unformatted_files() {
    // Joins real formatter exit/stderr -> FormatterResult::Failed -> snapshot tree and durable Done operation.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"save despite formatter failure\n");
    let (formatter, trace) = formatter_script(&f, b"printf 'formatter said no\\n' >&2\nexit 1\n");
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    let formatted = saved.formatted.as_ref().unwrap();
    assert_eq!(formatted.program, formatter.program);
    assert_eq!(formatted.args, formatter.args);
    assert_eq!(formatted.paths, vec![PathBuf::from("root.txt")]);
    assert_eq!(formatted.result, FormatterResult::Failed { code: Some(1), stderr: "formatter said no\n".into() });
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"save despite formatter failure\n");
    assert_eq!(fs::read(&trace).unwrap(), b"literal $(touch unexpected)\0--\0root.txt\0");
    assert_eq!(fs::read(trace.with_extension("count")).unwrap(), b"run\n");
}

#[test]
fn missing_formatter_is_spawn_failed_and_does_not_prevent_snapshot_or_log() {
    // Joins actual exec ENOENT -> listed SpawnFailed cause -> successful gitio/oplog snapshot.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"save despite missing program\n");
    let formatter = Formatter { program: f.common.join("not-installed-formatter"), args: vec!["--".into()], timeout: std::time::Duration::from_secs(60) };
    assert!(!formatter.program.exists());
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    let formatted = saved.formatted.as_ref().unwrap();
    assert_eq!(formatted.program, formatter.program);
    assert_eq!(formatted.args, formatter.args);
    assert_eq!(formatted.paths, vec![PathBuf::from("root.txt")]);
    match &formatted.result {
        FormatterResult::SpawnFailed { error } => assert!(error.contains("No such file or directory"), "{error}"),
        other => panic!("expected missing-program SpawnFailed, got {other:?}"),
    }
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"save despite missing program\n");
}

#[test]
fn clean_folder_has_no_formatter_invocation_but_first_snapshot_is_written() {
    // Joins real clean git status -> no eligible argv/no child -> first snapshot still creates the snap ref.
    let f = Fixture::new(&[]);
    let (formatter, trace) = formatter_script(&f, b"exit 77\n");
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(saved.formatted, None);
    assert!(!trace.exists());
    assert!(!trace.with_extension("count").exists());
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"base\n");
}

#[test]
fn no_formatter_skips_git_status_and_returns_none_while_saving_changed_files() {
    // Joins snapshot_formatted(None) -> no formatter status dependency -> real tree/oplog save.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"no formatter configured\n");
    let shim = f.common.join("refuse-status.sh");
    let marker = f.common.join("unexpected-status-call");
    write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"$3\" = status ]; then printf 'called\\n' >\"$MORTAR_STATUS_MARKER\"; exit 71; fi\nexec git \"$@\"\n").unwrap();
    let mut git = f.git.clone();
    git.program = shim;
    git.env.push(("MORTAR_STATUS_MARKER".into(), marker.clone().into_os_string()));
    let saved = snapshot_formatted(&git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP, None).unwrap();
    assert_snapshot_written(&f, &saved, &f.head);
    assert_eq!(saved.formatted, None);
    assert!(!marker.exists());
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"no formatter configured\n");
}
