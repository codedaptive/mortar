//! keep_plan: a missing attempt (an orphan k included) refused, then any orphan folder, then unsaved dropped
//! attempts (folder or not); the kept tip, the folders to remove (dropped attempts', the kept one last) and every
//! branch to delete, in k order.

use std::path::PathBuf;

use super::{keep_plan, Attempt, Kept};
use crate::test_ref_pairs::oid;
use crate::FolderError;

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333";

fn attempt(k: u32, hex: &str, folder: Option<&str>) -> Attempt {
    Attempt {
        k,
        branch: format!("refs/heads/try/z/r/{k}"),
        tip: oid(hex),
        folder: folder.map(PathBuf::from),
    }
}

fn three() -> Vec<Attempt> {
    vec![attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2")), attempt(3, C, Some("/t/try-3"))]
}

#[test]
fn keeps_attempt_2_removing_it_last_and_deleting_every_branch() {
    let kept = keep_plan("try/z/r/2", 2, &three(), &[], &[(1, true), (3, true)]).unwrap();
    assert_eq!(
        kept,
        Kept {
            tip: oid(B),
            remove: vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3"), PathBuf::from("/t/try-2")],
            delete: vec![
                ("refs/heads/try/z/r/1".to_string(), oid(A)),
                ("refs/heads/try/z/r/2".to_string(), oid(B)),
                ("refs/heads/try/z/r/3".to_string(), oid(C)),
            ],
        }
    );
}

#[test]
fn plans_in_k_order_whatever_the_input_order() {
    let attempts = vec![attempt(3, C, Some("/t/try-3")), attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2"))];
    let kept = keep_plan("try/z/r/1", 1, &attempts, &[], &[(3, true), (2, true)]).unwrap();
    assert_eq!(kept.remove, vec![PathBuf::from("/t/try-2"), PathBuf::from("/t/try-3"), PathBuf::from("/t/try-1")]);
    assert_eq!(
        kept.delete,
        vec![
            ("refs/heads/try/z/r/1".to_string(), oid(A)),
            ("refs/heads/try/z/r/2".to_string(), oid(B)),
            ("refs/heads/try/z/r/3".to_string(), oid(C)),
        ]
    );
}

#[test]
fn a_missing_attempt_is_refused() {
    match keep_plan("try/z/r/4", 4, &three(), &[], &[(1, true), (2, true), (3, true)]) {
        Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/4"),
        other => panic!("expected AttemptMissing, got {other:?}"),
    }
}

#[test]
fn the_missing_attempt_is_refused_before_unsaved_ones() {
    match keep_plan("try/z/r/9", 9, &three(), &[], &[]) {
        Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/9"),
        other => panic!("expected AttemptMissing, got {other:?}"),
    }
}

#[test]
fn unlisted_and_false_dropped_attempts_are_refused_in_k_order() {
    match keep_plan("try/z/r/2", 2, &three(), &[], &[(3, false)]) {
        Err(FolderError::AttemptsUnsaved { paths }) => {
            assert_eq!(paths, vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3")])
        }
        other => panic!("expected AttemptsUnsaved, got {other:?}"),
    }
}

#[test]
fn one_unsaved_dropped_attempt_is_refused_alone() {
    match keep_plan("try/z/r/1", 1, &three(), &[], &[(2, true), (3, false)]) {
        Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-3")]),
        other => panic!("expected AttemptsUnsaved, got {other:?}"),
    }
}

#[test]
fn the_kept_attempt_needs_no_saved_entry() {
    let kept = keep_plan("try/z/r/3", 3, &three(), &[], &[(1, true), (2, true), (3, false)]).unwrap();
    assert_eq!(kept.tip, oid(C));
}

#[test]
fn attempts_without_folders_are_skipped_from_remove_but_need_a_saved_entry() {
    let attempts = vec![attempt(1, A, None), attempt(2, B, None), attempt(3, C, Some("/t/try-3"))];
    let kept = keep_plan("try/z/r/2", 2, &attempts, &[], &[(1, true), (3, true)]).unwrap();
    assert_eq!(
        kept,
        Kept {
            tip: oid(B),
            remove: vec![PathBuf::from("/t/try-3")],
            delete: vec![
                ("refs/heads/try/z/r/1".to_string(), oid(A)),
                ("refs/heads/try/z/r/2".to_string(), oid(B)),
                ("refs/heads/try/z/r/3".to_string(), oid(C)),
            ],
        }
    );
}

#[test]
fn a_folderless_dropped_attempt_with_no_entry_is_refused_naming_its_branch() {
    let attempts = vec![attempt(1, A, None), attempt(2, B, None), attempt(3, C, Some("/t/try-3"))];
    match keep_plan("try/z/r/2", 2, &attempts, &[], &[]) {
        Err(FolderError::AttemptsUnsaved { paths }) => {
            assert_eq!(paths, vec![PathBuf::from("refs/heads/try/z/r/1"), PathBuf::from("/t/try-3")])
        }
        other => panic!("expected AttemptsUnsaved, got {other:?}"),
    }
}

#[test]
fn a_lone_attempt_is_kept_with_nothing_dropped() {
    let kept = keep_plan("try/z/r/1", 1, &[attempt(1, A, Some("/t/try-1"))], &[], &[]).unwrap();
    assert_eq!(
        kept,
        Kept {
            tip: oid(A),
            remove: vec![PathBuf::from("/t/try-1")],
            delete: vec![("refs/heads/try/z/r/1".to_string(), oid(A))],
        }
    );
}

#[test]
fn a_saved_orphan_is_still_refused_naming_its_folder() {
    let orphans = vec![(4, PathBuf::from("/t/try-4"))];
    match keep_plan("try/z/r/2", 2, &three(), &orphans, &[(1, true), (3, true), (4, true)]) {
        Err(FolderError::OrphanAttempt { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-4")]),
        other => panic!("expected OrphanAttempt, got {other:?}"),
    }
}

#[test]
fn orphans_are_refused_before_unsaved_attempts_in_k_order() {
    let orphans = vec![(5, PathBuf::from("/t/try-5")), (4, PathBuf::from("/t/try-4"))];
    match keep_plan("try/z/r/2", 2, &three(), &orphans, &[]) {
        Err(FolderError::OrphanAttempt { paths }) => {
            assert_eq!(paths, vec![PathBuf::from("/t/try-4"), PathBuf::from("/t/try-5")])
        }
        other => panic!("expected OrphanAttempt, got {other:?}"),
    }
}

#[test]
fn keeping_an_orphan_k_is_refused_as_missing_not_as_an_orphan() {
    let orphans = vec![(4, PathBuf::from("/t/try-4")), (5, PathBuf::from("/t/try-5"))];
    match keep_plan("try/z/r/4", 4, &three(), &orphans, &[(1, true), (2, true), (3, true)]) {
        Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/4"),
        other => panic!("expected AttemptMissing, got {other:?}"),
    }
}
