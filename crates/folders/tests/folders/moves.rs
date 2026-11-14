//! WO-orc-folders-it-2 #891, SPEC-folders WO-folders-4/S-49 and the #880 crash gap.
//! Joins folders::move_folder -> gitio::folder_move -> real sparse index/files and oplog::apply.
use super::move_support::{assert_move_op, caller_move, MoveFixture, BRANCH};
use crate::support::{run, write};
use folders::FolderError;
use gitio::{Git, Oid};
use oplog::{decode_lock_record, read_op_log, HeldLockError, KillPidProbe, OpPhase, OpResult, PidProbe};
use std::{fs, os::unix::process::ExitStatusExt, path::PathBuf};
use testkit::{run_with_kill_gate, test_child, write_git_shim};

#[test]
fn unsaved_folder_is_refused_with_save_next_step_and_no_ref_index_file_or_log_change() {
    // Public move_folder -> typed saved=false refusal before gitio, under the real caller lock.
    let m = MoveFixture::new();
    write(&m.folder, "kit/zone/file", b"actual unsaved local work\n");
    let before = m.observe();
    let raw_index = fs::read(m.git_dir.join("index")).unwrap();
    let error = m.move_saved(false).unwrap_err();
    let HeldLockError::Work(error) = error else { panic!("expected work refusal: {error:?}") };
    let text = error.to_string();
    assert!(text.contains("mortar save"), "{text}");
    assert!(matches!(error, FolderError::NotSaved { path } if path == m.folder));
    assert_eq!(m.observe(), before);
    assert_eq!(fs::read(m.git_dir.join("index")).unwrap(), raw_index);
}

#[test]
fn branch_already_at_new_gets_its_stale_files_and_index_moved_without_another_operation() {
    // S-47/S-49 caller already moved the real branch; folders only catches up index/files.
    let m = MoveFixture::new();
    run(&m.f.git, &["update-ref", BRANCH, m.new.as_str(), m.f.head.as_str()]);
    assert_eq!(run(&m.local, &["write-tree"]), run(&m.f.git, &["rev-parse", &format!("{}^{{tree}}", m.f.head)]));
    assert_eq!(fs::read(m.folder.join("kit/zone/file")).unwrap(), b"zone\n");
    let refs = m.refs();
    let log = m.log();
    let record = fs::read(m.git_dir.join("mortar-folder.json")).unwrap();
    m.move_saved(true).unwrap();
    m.assert_moved();
    assert_eq!(m.refs(), refs);
    assert_eq!(m.log(), log);
    assert_eq!(fs::read(m.git_dir.join("mortar-folder.json")).unwrap(), record);
    m.move_saved(true).unwrap();
    m.assert_moved();
    assert_eq!(m.refs(), refs);
    assert_eq!(m.log(), log);
}

#[test]
fn branch_at_old_moves_only_the_request_branch_with_one_exact_folder_move_operation_and_repeat_is_inert() {
    // Public move_folder -> real read-tree -> oplog Intent/Done/CAS, preserving unrelated refs and folder record.
    let m = MoveFixture::new();
    let before_records = read_op_log(&m.f.common).unwrap().records.len();
    let mut expected_refs = m.refs();
    expected_refs.insert(BRANCH.into(), m.new.to_string());
    let record = fs::read(m.git_dir.join("mortar-folder.json")).unwrap();
    m.move_saved(true).unwrap();
    m.assert_moved();
    assert_eq!(m.refs(), expected_refs);
    assert_eq!(fs::read(m.git_dir.join("mortar-folder.json")).unwrap(), record);
    let records = read_op_log(&m.f.common).unwrap().records;
    assert_move_op(&records[before_records..], &m.f.head, &m.new);
    let after = m.observe();
    m.move_saved(true).unwrap();
    m.assert_moved();
    assert_eq!(m.observe(), after);
}

#[test]
fn local_unstaged_staged_and_untracked_obstructions_refuse_without_branch_entry_file_or_log_changes() {
    // Even when the caller passes saved=true, real gitio read-tree must refuse work it would overwrite.
    for obstruction in ["unstaged", "staged", "untracked"] {
        let m = MoveFixture::new();
        let blocked = if obstruction == "untracked" { "kit/zone/new" } else { "kit/zone/file" };
        write(&m.folder, blocked, b"must not be overwritten\n");
        if obstruction == "staged" { run(&m.local, &["add", "--", blocked]); }
        let prefix = match obstruction { "unstaged" => " M", "staged" => "M ", _ => "??" };
        assert_eq!(run(&m.local, &["status", "--porcelain", "--", blocked]), format!("{prefix} {blocked}\n").as_bytes(), "{obstruction} fixture precondition");
        let before = m.observe();
        let error = m.move_saved(true).expect_err(&format!("real {obstruction} obstruction must refuse without overwriting {blocked}"));
        match error {
            HeldLockError::Work(FolderError::MoveRefused { path, paths }) => {
                assert_eq!(path, m.folder);
                assert_eq!(paths, vec![PathBuf::from(blocked)], "{obstruction}");
            }
            other => panic!("expected MoveRefused for {obstruction}: {other:?}"),
        }
        // Compare semantic index entries: gitio legitimately refreshes index stat/cache data before refusing.
        assert_eq!(m.observe(), before, "{obstruction}");
        assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), Some(m.f.head.clone()));
    }
}

#[test]
fn ignored_obstruction_is_replaced_by_the_target_blob_while_unrelated_ignored_work_survives() {
    // #1179/S-55: move_folder -> gitio protection skips ignored files, then real sparse move installs target.
    let m = MoveFixture::new();
    fs::write(m.f.common.join("info/exclude"), b"/kit/zone/new\n/local-ignored\n").unwrap();
    write(&m.folder, "kit/zone/new", b"ignored bytes may be replaced\n");
    write(&m.folder, "local-ignored", b"unrelated ignored bytes stay\n");
    assert_eq!(run(&m.local, &["check-ignore", "--", "kit/zone/new"]), b"kit/zone/new\n");
    assert!(run(&m.local, &["status", "--porcelain", "--", "kit/zone/new"]).is_empty());
    let start = read_op_log(&m.f.common).unwrap().records.len();
    m.move_saved(true).unwrap();
    m.assert_moved();
    assert_eq!(fs::read(m.folder.join("kit/zone/new")).unwrap(), b"new path bytes\n");
    assert_eq!(fs::read(m.folder.join("local-ignored")).unwrap(), b"unrelated ignored bytes stay\n");
    assert_move_op(&read_op_log(&m.f.common).unwrap().records[start..], &m.f.head, &m.new);
    let after = m.observe();
    m.move_saved(true).unwrap();
    assert_eq!(m.observe(), after);
}

#[test]
fn tracked_local_edits_refuse_the_move_without_ref_index_file_or_log_changes() {
    // WO-orc-folders-it-3 #975, move_folder case 4: folders -> real gitio refusal -> no oplog operation.
    // saved=true deliberately reaches gitio's protection, rather than the early NotSaved guard.
    for staged in [false, true] {
        let m = MoveFixture::new();
        let path = "kit/zone/file";
        let edit = b"tracked local edit must survive\n";
        write(&m.folder, path, edit);
        if staged { run(&m.local, &["add", "--", path]); }
        let status = if staged { "M  kit/zone/file\n" } else { " M kit/zone/file\n" };
        assert_eq!(run(&m.local, &["status", "--porcelain", "--", path]), status.as_bytes());
        assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), Some(m.f.head.clone()));
        let before = m.observe();
        let error = m.move_saved(true).expect_err("a tracked edit in the move's way must refuse");
        match error {
            HeldLockError::Work(FolderError::MoveRefused { path: folder, paths }) => {
                assert_eq!(folder, m.folder);
                assert_eq!(paths, vec![PathBuf::from(path)], "staged={staged}");
            }
            other => panic!("expected MoveRefused for staged={staged}: {other:?}"),
        }
        // gitio may refresh index stat/cache bytes; staged entries, files, refs, folder record and log must not change.
        assert_eq!(m.observe(), before, "staged={staged}");
        assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), Some(m.f.head.clone()));
        assert_eq!(fs::read(m.folder.join(path)).unwrap(), edit);
    }
}

#[test]
fn killed_after_real_folder_move_before_branch_op_repeats_to_new_with_one_folder_move_operation() {
    // Real worker under its caller lock -> successful read-tree -> SIGKILL/reap -> caller takeover/recover/repeat.
    let m = MoveFixture::new();
    let before_log = m.log();
    let before_records = read_op_log(&m.f.common).unwrap().records.len();
    let mut expected_refs = m.refs();
    expected_refs.insert(BRANCH.into(), m.new.to_string());
    let marker = m.f.common.join("folder-move-boundary");
    let shim = m.f.common.join("kill-after-folder-move.sh");
    // Shared run_with_kill_gate owns the FIFO/wait/reaped-ack protocol; this boundary is read-tree, not update-ref.
    write_git_shim(&shim, br#"#!/bin/sh
set -eu
if [ "${3-}" != read-tree ] || [ "${4-}" != -m ]; then exec git "$@"; fi
[ "$PPID" = "$MORTAR_TEST_WORKER_PID" ] || exit 98
git "$@"
printf 'folder-moved-before-branch-op\n' > "$MORTAR_TEST_MOVE_MARKER"
printf 'armed\n' > "$TESTKIT_GATE_ARMED"
kill -KILL "$MORTAR_TEST_WORKER_PID"
IFS= read -r release < "$TESTKIT_GATE_RELEASE"
[ "$release" = release ] || exit 97
if kill -0 "$MORTAR_TEST_WORKER_PID" 2>/dev/null; then
  printf 'not-reaped\n' > "$TESTKIT_GATE_ACK"
else
  printf 'reaped\n' > "$TESTKIT_GATE_ACK"
fi
"#).unwrap();
    let mut command = test_child("moves::move_folder_crash_worker").unwrap();
    command.envs(m.f.git.env.iter().cloned());
    command.env("MORTAR_TEST_MOVE_REPO", &m.f.repo.path)
        .env("MORTAR_TEST_MOVE_FOLDER", &m.folder).env("MORTAR_TEST_MOVE_COMMON", &m.f.common)
        .env("MORTAR_TEST_MOVE_OLD", m.f.head.as_str()).env("MORTAR_TEST_MOVE_NEW", m.new.as_str())
        .env("MORTAR_TEST_MOVE_PROGRAM", &shim).env("MORTAR_TEST_MOVE_MARKER", &marker);
    let output = run_with_kill_gate(&mut command, &m.f.common).unwrap();
    assert_eq!(output.status.signal(), Some(9), "{output:?}");
    assert_eq!(fs::read(marker).unwrap(), b"folder-moved-before-branch-op\n");
    assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), Some(m.f.head.clone()));
    assert_eq!(run(&m.local, &["write-tree"]), run(&m.f.git, &["rev-parse", &format!("{}^{{tree}}", m.new)]));
    assert_eq!(fs::read(m.folder.join("kit/zone/file")).unwrap(), b"moved new bytes\n");
    assert_eq!(fs::read(m.folder.join("kit/zone/new")).unwrap(), b"new path bytes\n");
    assert!(!m.folder.join("kit/zone/conflict").exists());
    assert_eq!(m.log(), before_log, "crash was before any branch-op Intent, not merely before update-ref");
    let held = decode_lock_record(&fs::read_to_string(m.f.common.join("mortar/locks/join-red.lock")).unwrap()).unwrap();
    assert_eq!(held.key, "move-folder-caller");
    assert!(!KillPidProbe.pid_alive(held.pid), "worker was actually reaped");
    m.move_saved(true).unwrap();
    m.assert_moved();
    assert_eq!(m.refs(), expected_refs);
    let records = read_op_log(&m.f.common).unwrap().records;
    let added = &records[before_records..];
    assert_eq!(added.len(), 4, "one caller-lock takeover plus one folder-move, each Intent/Done");
    assert_eq!(added[0].kind, "lock-takeover");
    assert_eq!(added[0].phase, OpPhase::Intent);
    assert_eq!(added[1].kind, "lock-takeover");
    assert_eq!(added[1].phase, OpPhase::Done);
    assert_eq!(added[1].result, Some(OpResult::Moved));
    assert_move_op(&added[2..], &m.f.head, &m.new);
    let after = m.observe();
    m.move_saved(true).unwrap();
    m.assert_moved();
    assert_eq!(m.observe(), after);
}

#[test]
#[ignore = "actual killed folder-move worker, invoked and reaped by the crash parent"]
fn move_folder_crash_worker() {
    let path = |name| PathBuf::from(std::env::var_os(name).unwrap());
    let mut git = Git::new(path("MORTAR_TEST_MOVE_REPO"));
    git.program = path("MORTAR_TEST_MOVE_PROGRAM");
    git.env.push(("MORTAR_TEST_WORKER_PID".into(), std::process::id().to_string().into()));
    let old = Oid::parse(&std::env::var("MORTAR_TEST_MOVE_OLD").unwrap()).unwrap();
    let new = Oid::parse(&std::env::var("MORTAR_TEST_MOVE_NEW").unwrap()).unwrap();
    caller_move(&git, &path("MORTAR_TEST_MOVE_COMMON"), &path("MORTAR_TEST_MOVE_FOLDER"), &old, &new, true).unwrap();
    panic!("real worker must be killed before branch-op creation");
}
