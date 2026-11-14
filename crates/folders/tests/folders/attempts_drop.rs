//! drop_attempts in a throwaway repository: the cases src/attempts/drop_attempts.rs's header names (S-82), an orphan
//! attempt folder refused, and a drop after remake_orphans going through (S-96 (2), S-98).
use super::attempt_support::{trysnap_ref, try_ref, worktrees, Attempts, REQ_REF};
use super::maintenance_support::{caller, child, instrument, records, ACTOR};
use crate::support::{run, write};
use folders::FolderError;
use gitio::{Git, Oid};
use oplog::{OpPhase, OpResult, RefMove};
use std::{fs, path::PathBuf};

const ALL_SAVED: &[(u32, bool)] = &[(1, true), (2, true), (3, true)];
const TWO_SAVED: &[(u32, bool)] = &[(1, true), (2, true)];

fn drop_attempts(a: &Attempts, git: &Git, saved: &[(u32, bool)]) -> Result<Vec<u32>, FolderError> {
    caller(git, &a.f.common, || folders::drop_attempts(git, "red", "one", saved, ACTOR))
}
fn deleted(refname: &str, old: &Oid) -> RefMove {
    RefMove { refname: refname.into(), old: Some(old.clone()), new: Oid::parse(&"0".repeat(40)).unwrap() }
}

fn assert_attempts_gone(a: &Attempts) {
    for k in 1..=3 {
        assert!(!a.folder(k).exists(), "try-{k}");
        assert_eq!(a.f.git.read_ref(&try_ref(k)).unwrap(), None, "try {k}");
    }
    assert_eq!(worktrees(&a.f.git).len(), 2, "the main worktree and the request folder");
}

#[test]
fn drop_of_3_saved_attempts_removes_every_folder_and_branch_in_one_op_and_a_repeat_returns_empty_and_logs_nothing() {
    let a = Attempts::with(3);
    a.trysnaps(3);
    let tip = a.commit_in(2);
    let head = a.f.head.clone();
    let start = records(&a.f).len();
    assert_eq!(drop_attempts(&a, &a.f.git, ALL_SAVED).unwrap(), vec![1, 2, 3]);
    assert_attempts_gone(&a);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head.clone()), "the request branch does not move");
    for k in 1..=3 { assert_eq!(a.f.git.read_ref(&trysnap_ref(k)).unwrap(), Some(head.clone()), "trysnap {k}"); }
    let ops = records(&a.f);
    let ops = &ops[start..];
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].phase, OpPhase::Intent);
    assert_eq!(ops[0].result, None);
    assert_eq!(ops[1].phase, OpPhase::Done);
    assert_eq!(ops[1].result, Some(OpResult::Moved));
    assert_eq!(ops[0].op_id, ops[1].op_id);
    for op in ops {
        assert_eq!(op.kind, "folder-drop-attempts");
        assert_eq!(op.actor, ACTOR);
        assert_eq!(op.key, format!("folders:drop-attempts:{REQ_REF}:{}", op.op_id));
        assert_eq!(op.reason, format!("drop 3 attempts of {REQ_REF}"));
        assert_eq!(op.moves, [deleted(&try_ref(1), &head), deleted(&try_ref(2), &tip), deleted(&try_ref(3), &head)]);
    }
    let before = a.state();
    assert_eq!(drop_attempts(&a, &a.f.git, ALL_SAVED).unwrap(), Vec::<u32>::new());
    assert_eq!(a.state(), before);
}

#[test]
fn an_unsaved_attempt_with_a_folder_or_folderless_is_attempts_unsaved_with_nothing_changed() {
    let a = Attempts::with(3);
    let before = a.state();
    match drop_attempts(&a, &a.f.git, TWO_SAVED) {
        Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![a.folder(3)]),
        other => panic!("expected AttemptsUnsaved, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    run(&a.f.git, &["worktree", "remove", a.folder(3).to_str().unwrap()]);
    let before = a.state();
    match drop_attempts(&a, &a.f.git, TWO_SAVED) {
        Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![PathBuf::from(try_ref(3))]),
        other => panic!("expected AttemptsUnsaved, got {other:?}"),
    }
    assert_eq!(a.state(), before);
}

#[test]
fn an_untracked_file_is_gits_refusal_with_the_op_not_run_and_a_repeat_after_cleaning_finishes() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    write(&a.folder(3), "kit/zone/untracked", b"must survive\n");
    let start = records(&a.f).len();
    match drop_attempts(&a, &a.f.git, ALL_SAVED) {
        Err(FolderError::Git(error)) => assert!(error.stderr.contains("untracked"), "{error:?}"),
        other => panic!("expected git's refusal, got {other:?}"),
    }
    assert!(!a.folder(1).exists(), "folders are removed in k order");
    assert!(!a.folder(2).exists());
    assert_eq!(fs::read(a.folder(3).join("kit/zone/untracked")).unwrap(), b"must survive\n");
    for k in 1..=3 { assert_eq!(a.f.git.read_ref(&try_ref(k)).unwrap(), Some(head.clone()), "try {k}"); }
    assert_eq!(records(&a.f).len(), start);
    fs::remove_file(a.folder(3).join("kit/zone/untracked")).unwrap();
    assert_eq!(drop_attempts(&a, &a.f.git, ALL_SAVED).unwrap(), vec![1, 2, 3]);
    assert_attempts_gone(&a);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head));
}

#[test]
fn a_try_branch_moved_after_the_removals_is_attempt_refs_left_and_a_repeat_on_fresh_refs_finishes() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    let next = child(&a.f, "try 3 raced");
    let (git, marker, _) = instrument(&a.f, "move-ref", &try_ref(3), &next);
    match drop_attempts(&a, &git, ALL_SAVED) {
        Err(FolderError::AttemptRefsLeft { moved, at, removed }) => {
            assert_eq!(moved, try_ref(3));
            assert_eq!(at, Some(next.clone()));
            assert_eq!(removed, vec![a.folder(1), a.folder(2), a.folder(3)]);
        }
        other => panic!("expected AttemptRefsLeft, got {other:?}"),
    }
    assert_eq!(fs::read(marker).unwrap(), b"removed\n");
    for k in 1..=3 { assert!(!a.folder(k).exists(), "try-{k}"); }
    assert_eq!(a.f.git.read_ref(&try_ref(1)).unwrap(), Some(head.clone()));
    assert_eq!(a.f.git.read_ref(&try_ref(2)).unwrap(), Some(head.clone()));
    assert_eq!(a.f.git.read_ref(&try_ref(3)).unwrap(), Some(next));
    assert_eq!(drop_attempts(&a, &a.f.git, ALL_SAVED).unwrap(), vec![1, 2, 3]);
    assert_attempts_gone(&a);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head));
}

#[test]
fn an_orphan_attempt_folder_is_orphan_attempt_naming_its_folder_with_nothing_touched() {
    let a = Attempts::with(3);
    a.orphan(3);
    let before = a.state();
    match drop_attempts(&a, &a.f.git, ALL_SAVED) {
        Err(FolderError::OrphanAttempt { paths }) => assert_eq!(paths, vec![a.folder(3)]),
        other => panic!("expected OrphanAttempt, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    assert!(a.folder(3).exists());
}

#[test]
fn after_remake_orphans_a_drop_with_every_attempt_saved_finishes_and_remove_goes_ahead() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    a.trysnap(3, true);
    a.orphan(3);
    assert_eq!(a.remake().unwrap(), vec![a.folder(3)]);
    assert_eq!(drop_attempts(&a, &a.f.git, ALL_SAVED).unwrap(), vec![1, 2, 3]);
    assert_attempts_gone(&a);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head));
    a.remove_request().unwrap();
    assert!(!a.request.exists());
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), None);
}
