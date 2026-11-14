//! #1019: folders::remove -> real gitio worktree removal -> oplog transactional ref deletion/repeat.
use super::maintenance_support::*;
use crate::support::{run, setup, write};
use folders::{make_zone_folder, FolderError};
use std::{fs, path::PathBuf};

#[test]
fn joined_saved_remove_deletes_folder_branch_and_snapshot_in_one_exact_op_and_recreated_branch_repeats() {
    let m = Removal::new(); m.snapshot();
    let record = fs::read(m.git_dir.join("mortar-folder.json")).unwrap();
    let mut expected_refs = refs(&m.f.git); expected_refs.remove(BRANCH); expected_refs.remove(SNAP);
    let start = records(&m.f).len();
    m.remove(&m.f.git).unwrap();
    assert!(!m.folder.exists());
    assert_eq!(refs(&m.f.git), expected_refs);
    let ops = records(&m.f);
    assert_delete_pair(&ops[start..], "folder-remove", &[BRANCH, SNAP], &m.f.head);
    assert_eq!(ops[start].key, format!("folders:remove:{BRANCH}@{}", m.f.head));
    assert_eq!(ops[start].reason, format!("remove folder branch {BRANCH}"));
    recreate(&m, &record); m.snapshot();
    let start = records(&m.f).len();
    m.remove(&m.f.git).unwrap();
    assert!(!m.folder.exists());
    assert_eq!(refs(&m.f.git), expected_refs);
    let ops = records(&m.f);
    assert_delete_pair(&ops[start..], "folder-remove", &[BRANCH, SNAP], &m.f.head);
    assert_eq!(ops[start].key, format!("folders:remove:{BRANCH}@{}:{}", m.f.head, ops[start].op_id));
}

#[test]
fn unsaved_refused_files_and_invalid_folder_states_preserve_real_work_refs_index_record_and_log() {
    for case in ["unsaved", "cap", "not-joined", "detached", "unborn", "no-record", "wrong-branch", "missing-zone", "untracked"] {
        let m = Removal::new(); m.snapshot();
        match case {
            "not-joined" => { let next = child(&m.f, "request not in zone"); run(&m.f.git, &["update-ref", BRANCH, next.as_str()]); }
            "detached" => { run(&m.local, &["checkout", "--detach", "HEAD"]); }
            "unborn" => { run(&m.local, &["symbolic-ref", "HEAD", "refs/heads/unborn"]); }
            "no-record" => { fs::remove_file(m.git_dir.join("mortar-folder.json")).unwrap(); }
            "wrong-branch" => { run(&m.local, &["checkout", "-b", "wrong-branch"]); }
            "missing-zone" => { run(&m.f.git, &["update-ref", "-d", "refs/heads/zone/red"]); }
            "untracked" => { write(&m.folder, "local-only", b"must survive\n"); }
            _ => {}
        }
        let before = observe(&m);
        let refused = if case == "cap" { vec![PathBuf::from("large-file")] } else { vec![] };
        let error = caller(&m.f.git, &m.f.common, || folders::remove(&m.f.git, &m.folder, case != "unsaved", &refused, Some(17), ACTOR)).expect_err(case);
        match (case, error) {
            ("unsaved" | "cap", FolderError::NotRemovedUnsaved { path, refused: actual, cap }) => {
                assert_eq!(path, m.folder); assert_eq!(actual, refused); assert_eq!(cap, Some(17));
            }
            ("not-joined", FolderError::NotJoined { path, branch, zone_branch }) => {
                assert_eq!(path, m.folder); assert_eq!(branch, BRANCH); assert_eq!(zone_branch, "refs/heads/zone/red");
            }
            ("detached" | "unborn" | "no-record" | "wrong-branch", FolderError::NotAFolder { path, reason }) => { assert_eq!(path, m.folder); assert!(!reason.is_empty()); }
            ("missing-zone", FolderError::ZoneBranchMissing { branch }) => assert_eq!(branch, "zone/red"),
            ("untracked", FolderError::Git(error)) => { assert!(error.code.is_some_and(|c| c != 0)); assert!(error.stderr.contains("untracked"), "{error:?}"); }
            (_, other) => panic!("{case}: wrong refusal {other:?}"),
        }
        assert_eq!(observe(&m), before, "{case}");
    }
}

#[test]
fn zone_folder_refuses_without_touching_its_record_files_or_log() {
    let m = Removal::new();
    let zone = m.f.repo.path.join("standing-zone");
    make_zone_folder(&setup(&m.f.git, &zone), "red", &["kit/zone".into()], None).unwrap();
    let record_path = m.f.git.git_dir(&zone).unwrap().join("mortar-folder.json");
    let record = fs::read(&record_path).unwrap(); let before_refs = refs(&m.f.git); let before_log = log(&m.f);
    let error = caller(&m.f.git, &m.f.common, || folders::remove(&m.f.git, &zone, true, &[], None, ACTOR)).unwrap_err();
    assert!(matches!(error, FolderError::ZoneFolder { path, zone: name } if path == zone && name == "red"));
    assert_eq!(fs::read(&record_path).unwrap(), record);
    assert_eq!(fs::read(zone.join("kit/zone/file")).unwrap(), b"zone\n");
    assert_eq!(refs(&m.f.git), before_refs); assert_eq!(log(&m.f), before_log);
}

#[test]
fn snapshot_raced_after_actual_worktree_removal_keeps_both_refs_and_sweep_finishes_leftovers() {
    let m = Removal::new(); m.snapshot();
    let next = child(&m.f, "concurrent snapshot");
    let (git, marker, _) = instrument(&m.f, "move-ref", SNAP, &next);
    let error = m.remove(&git).unwrap_err();
    assert!(matches!(&error, FolderError::RemovedRefsLeft { path, branch, moved, at } if path == &m.folder && branch == BRANCH && moved == SNAP && at == &Some(next.clone())), "{error:?}");
    assert!(error.to_string().contains("mortar sweep"));
    assert_eq!(fs::read(marker).unwrap(), b"removed\n"); assert!(!m.folder.exists());
    assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), Some(m.f.head.clone()));
    assert_eq!(m.f.git.read_ref(SNAP).unwrap(), Some(next));
    let start = records(&m.f).len();
    let deleted = sweep(&m.f, &m.f.git, &[], &[]).unwrap();
    assert_eq!(deleted, [BRANCH.to_string(), SNAP.to_string()]);
    assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), None); assert_eq!(m.f.git.read_ref(SNAP).unwrap(), None);
    let ops = records(&m.f); assert_eq!(ops.len(), start + 2);
    assert_eq!(ops[start].kind, "folder-sweep-refs"); assert_eq!(ops[start + 1].result, Some(oplog::OpResult::Moved));
}

#[test]
fn repeated_remove_with_branch_moved_after_worktree_removal_refuses_and_keeps_the_refs() {
    let m = Removal::new(); m.snapshot();
    let record = fs::read(m.git_dir.join("mortar-folder.json")).unwrap();
    m.remove(&m.f.git).unwrap(); recreate(&m, &record); m.snapshot();
    let before_log = log(&m.f); let next = child(&m.f, "branch raced on repeat");
    let (git, _, _) = instrument(&m.f, "move-ref", BRANCH, &next);
    let error = m.remove(&git).unwrap_err();
    assert!(matches!(error, FolderError::RemovedRefsLeft { branch, moved, at, .. } if branch == BRANCH && moved == BRANCH && at == Some(next.clone())));
    assert!(!m.folder.exists()); assert_eq!(m.f.git.read_ref(BRANCH).unwrap(), Some(next));
    assert_eq!(m.f.git.read_ref(SNAP).unwrap(), Some(m.f.head.clone()));
    assert_eq!(log(&m.f), before_log, "Repeated appends no second removal Intent when the branch is elsewhere");
}

#[test]
fn post_worktree_removal_git_and_oplog_errors_are_typed_without_deleting_either_ref() {
    for action in ["git-error", "oplog-error"] {
        let m = Removal::new(); m.snapshot();
        let before_refs = refs(&m.f.git);
        let (git, marker, _) = instrument(&m.f, action, SNAP, &m.f.head);
        let error = m.remove(&git).expect_err(action);
        match (action, error) {
            ("git-error", FolderError::Git(error)) => { assert_eq!(error.code, Some(67)); assert!(error.stderr.contains("fixture common-dir error")); }
            ("oplog-error", FolderError::Oplog(_)) => {}
            (_, other) => panic!("{action}: wrong error {other:?}"),
        }
        assert_eq!(fs::read(marker).unwrap(), b"removed\n"); assert!(!m.folder.exists());
        assert_eq!(refs(&m.f.git), before_refs);
    }
}

fn recreate(m: &Removal, record: &[u8]) {
    run(&m.f.git, &["update-ref", BRANCH, m.f.head.as_str()]);
    run(&m.f.git, &["worktree", "add", m.folder.to_str().unwrap(), "req/red/one"]);
    fs::write(m.f.git.git_dir(&m.folder).unwrap().join("mortar-folder.json"), record).unwrap();
}
