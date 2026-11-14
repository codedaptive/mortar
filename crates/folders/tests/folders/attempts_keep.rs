//! keep in a throwaway repository: the cases src/attempts/keep.rs's header names, orphan attempt folders refused and
//! then kept past after remake_orphans (S-96 (2), S-98), and AttemptRefsLeft from a ref moved between keep's read and
//! its op (S-69, S-93).
use super::attempt_support::{trysnap_ref, try_ref, worktrees, Attempts, REQ_REF};
use super::maintenance_support::{child, instrument, records, ACTOR};
use crate::support::{run, write};
use folders::FolderError;
use gitio::Oid;
use oplog::{OpPhase, OpResult, RefMove};
use std::fs;

const ALL_SAVED: &[(u32, bool)] = &[(1, true), (3, true)];

fn zero() -> Oid { Oid::parse(&"0".repeat(40)).unwrap() }
fn mv(refname: &str, old: &Oid, new: &Oid) -> RefMove {
    RefMove { refname: refname.into(), old: Some(old.clone()), new: new.clone() }
}

/// The folder-keep op's Intent and Done records from `start`, keyed and moved as keep_op's contract says.
fn assert_keep_op(a: &Attempts, start: usize, tip: &Oid, moves: &[RefMove]) {
    let ops = records(&a.f);
    let ops = &ops[start..];
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].phase, OpPhase::Intent);
    assert_eq!(ops[0].result, None);
    assert_eq!(ops[1].phase, OpPhase::Done);
    assert_eq!(ops[1].result, Some(OpResult::Moved));
    assert_eq!(ops[0].op_id, ops[1].op_id);
    for op in ops {
        assert_eq!(op.kind, "folder-keep");
        assert_eq!(op.actor, ACTOR);
        assert_eq!(op.key, format!("folders:keep:{REQ_REF}@{tip}"));
        assert_eq!(op.reason, format!("keep {tip} on {REQ_REF}"));
        assert_eq!(op.moves, moves);
    }
}

fn assert_attempts_gone(a: &Attempts) {
    for k in 1..=3 {
        assert!(!a.dir.join(format!("try-{k}")).exists(), "try-{k}");
        assert_eq!(a.f.git.read_ref(&try_ref(k)).unwrap(), None, "try {k}");
    }
    assert_eq!(worktrees(&a.f.git).len(), 2, "the main worktree and the request folder");
}

#[test]
fn keep_2_of_3_saved_moves_the_request_to_try_2_clears_every_attempt_in_one_op_keeps_trysnaps_and_a_repeat_is_missing() {
    let a = Attempts::with(3);
    a.trysnaps(3);
    let tip = a.commit_in(2);
    let head = a.f.head.clone();
    let start = records(&a.f).len();
    assert_eq!(a.keep(&a.f.git, 2, ALL_SAVED).unwrap(), (head.clone(), tip.clone()));
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(tip.clone()));
    assert_attempts_gone(&a);
    for k in 1..=3 { assert_eq!(a.f.git.read_ref(&trysnap_ref(k)).unwrap(), Some(head.clone()), "trysnap {k}"); }
    assert_keep_op(&a, start, &tip, &[
        mv(REQ_REF, &head, &tip), mv(&try_ref(1), &head, &zero()), mv(&try_ref(2), &tip, &zero()), mv(&try_ref(3), &head, &zero()),
    ]);
    let before = a.state();
    match a.keep(&a.f.git, 2, ALL_SAVED) {
        Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/red/one/2"),
        other => panic!("expected AttemptMissing, got {other:?}"),
    }
    assert_eq!(a.state(), before);
}

#[test]
fn an_unsaved_dropped_attempt_is_attempts_unsaved_naming_its_folder_with_nothing_changed() {
    let a = Attempts::with(3);
    let before = a.state();
    for saved in [&[(1, true)][..], &[(1, true), (3, false)][..]] {
        match a.keep(&a.f.git, 2, saved) {
            Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![a.dir.join("try-3")], "{saved:?}"),
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
        assert_eq!(a.state(), before, "{saved:?}");
    }
}

#[test]
fn a_request_branch_moved_on_is_request_moved_on_with_nothing_changed() {
    let a = Attempts::with(3);
    run(&a.f.git, &["update-ref", REQ_REF, child(&a.f, "request moves on").as_str()]);
    let before = a.state();
    match a.keep(&a.f.git, 2, ALL_SAVED) {
        Err(FolderError::RequestMovedOn { branch, attempt }) => {
            assert_eq!(branch, "req/red/one");
            assert_eq!(attempt, "try/red/one/2");
        }
        other => panic!("expected RequestMovedOn, got {other:?}"),
    }
    assert_eq!(a.state(), before);
}

#[test]
fn an_untracked_file_is_gits_refusal_with_the_op_not_run_and_a_repeat_after_cleaning_finishes() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    write(&a.folder(3), "kit/zone/untracked", b"must survive\n");
    let start = records(&a.f).len();
    match a.keep(&a.f.git, 2, ALL_SAVED) {
        Err(FolderError::Git(error)) => assert!(error.stderr.contains("untracked"), "{error:?}"),
        other => panic!("expected git's refusal, got {other:?}"),
    }
    assert!(!a.dir.join("try-1").exists(), "try-1 is removed before try-3, the kept try-2 last");
    assert!(a.dir.join("try-2").exists());
    assert_eq!(fs::read(a.dir.join("try-3/kit/zone/untracked")).unwrap(), b"must survive\n");
    for k in 1..=3 { assert_eq!(a.f.git.read_ref(&try_ref(k)).unwrap(), Some(head.clone()), "try {k}"); }
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head.clone()));
    assert_eq!(records(&a.f).len(), start);
    fs::remove_file(a.dir.join("try-3/kit/zone/untracked")).unwrap();
    assert_eq!(a.keep(&a.f.git, 2, ALL_SAVED).unwrap(), (head.clone(), head.clone()));
    assert_attempts_gone(&a);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head));
}

#[test]
fn an_orphan_attempt_folder_is_orphan_attempt_naming_its_folder_with_nothing_touched() {
    let a = Attempts::with(3);
    a.orphan(3);
    let before = a.state();
    match a.keep(&a.f.git, 2, ALL_SAVED) {
        Err(FolderError::OrphanAttempt { paths }) => assert_eq!(paths, vec![a.dir.join("try-3")]),
        other => panic!("expected OrphanAttempt, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    assert!(a.dir.join("try-3").exists());
}

#[test]
fn keep_of_an_orphan_k_is_attempt_missing_with_nothing_touched() {
    let a = Attempts::with(3);
    a.orphan(2);
    let before = a.state();
    match a.keep(&a.f.git, 2, ALL_SAVED) {
        Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/red/one/2"),
        other => panic!("expected AttemptMissing, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    assert!(a.dir.join("try-2").exists());
}

#[test]
fn after_remake_orphans_keep_of_another_attempt_with_every_dropped_one_saved_finishes_and_remove_goes_ahead() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    a.trysnap(3, true);
    a.orphan(3);
    assert_eq!(a.remake().unwrap(), vec![a.dir.join("try-3")]);
    assert_eq!(a.keep(&a.f.git, 2, ALL_SAVED).unwrap(), (head.clone(), head.clone()));
    assert_attempts_gone(&a);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head));
    a.remove_request().unwrap();
    assert!(!a.request.exists());
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), None);
}

#[test]
fn a_try_branch_moved_after_the_removals_is_attempt_refs_left_and_a_repeat_on_fresh_refs_finishes() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    let next = child(&a.f, "try 3 raced");
    let (git, marker, _) = instrument(&a.f, "move-ref", &try_ref(3), &next);
    match a.keep(&git, 2, ALL_SAVED) {
        Err(FolderError::AttemptRefsLeft { moved, at, removed }) => {
            assert_eq!(moved, try_ref(3));
            assert_eq!(at, Some(next.clone()));
            assert_eq!(removed, vec![a.dir.join("try-1"), a.dir.join("try-3"), a.dir.join("try-2")]);
        }
        other => panic!("expected AttemptRefsLeft, got {other:?}"),
    }
    assert_eq!(fs::read(marker).unwrap(), b"removed\n");
    for k in 1..=3 { assert!(!a.dir.join(format!("try-{k}")).exists(), "try-{k}"); }
    assert_eq!(a.f.git.read_ref(&try_ref(1)).unwrap(), Some(head.clone()));
    assert_eq!(a.f.git.read_ref(&try_ref(2)).unwrap(), Some(head.clone()));
    assert_eq!(a.f.git.read_ref(&try_ref(3)).unwrap(), Some(next));
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head.clone()));
    assert_eq!(a.keep(&a.f.git, 2, ALL_SAVED).unwrap(), (head.clone(), head.clone()));
    assert_attempts_gone(&a);
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(head));
}

#[test]
fn the_request_branch_moved_after_the_removals_is_attempt_refs_left_and_a_repeat_refuses_afresh_as_moved_on() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    let next = child(&a.f, "request raced");
    let (git, _, _) = instrument(&a.f, "move-ref", REQ_REF, &next);
    match a.keep(&git, 2, ALL_SAVED) {
        Err(FolderError::AttemptRefsLeft { moved, at, removed }) => {
            assert_eq!(moved, REQ_REF);
            assert_eq!(at, Some(next.clone()));
            assert_eq!(removed, vec![a.dir.join("try-1"), a.dir.join("try-3"), a.dir.join("try-2")]);
        }
        other => panic!("expected AttemptRefsLeft, got {other:?}"),
    }
    for k in 1..=3 {
        assert!(!a.dir.join(format!("try-{k}")).exists(), "try-{k}");
        assert_eq!(a.f.git.read_ref(&try_ref(k)).unwrap(), Some(head.clone()), "try {k}");
    }
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(next.clone()));
    let before = a.state();
    match a.keep(&a.f.git, 2, ALL_SAVED) {
        Err(FolderError::RequestMovedOn { branch, attempt }) => {
            assert_eq!(branch, "req/red/one");
            assert_eq!(attempt, "try/red/one/2");
        }
        other => panic!("expected RequestMovedOn, got {other:?}"),
    }
    assert_eq!(a.state(), before);
}
