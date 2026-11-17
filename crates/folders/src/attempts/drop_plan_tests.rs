//! drop_plan: any orphan folder refused first, all named in k order, saved or not; then unsaved attempts
//! (folder or not) refused in k order; every attempt folder to remove and every branch to delete, in k order; no
//! attempts plan nothing.

use std::path::PathBuf;

use super::{drop_plan, Attempt, Dropped};
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

#[test]
fn drops_three_saved_attempts_in_k_order_whatever_the_input_order() {
    let attempts = vec![attempt(3, C, Some("/t/try-3")), attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2"))];
    let dropped = drop_plan(&attempts, &[], &[(3, true), (1, true), (2, true)]).unwrap();
    assert_eq!(
        dropped,
        Dropped {
            remove: vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-2"), PathBuf::from("/t/try-3")],
            delete: vec![
                ("refs/heads/try/z/r/1".to_string(), oid(A)),
                ("refs/heads/try/z/r/2".to_string(), oid(B)),
                ("refs/heads/try/z/r/3".to_string(), oid(C)),
            ],
        }
    );
}

#[test]
fn attempts_without_folders_are_skipped_from_remove_but_deleted() {
    let attempts = vec![attempt(1, A, None), attempt(2, B, Some("/t/try-2"))];
    let dropped = drop_plan(&attempts, &[], &[(1, true), (2, true)]).unwrap();
    assert_eq!(
        dropped,
        Dropped {
            remove: vec![PathBuf::from("/t/try-2")],
            delete: vec![("refs/heads/try/z/r/1".to_string(), oid(A)), ("refs/heads/try/z/r/2".to_string(), oid(B))],
        }
    );
}

#[test]
fn unlisted_and_false_attempts_are_refused_in_k_order() {
    let attempts = vec![attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2")), attempt(3, C, Some("/t/try-3"))];
    match drop_plan(&attempts, &[], &[(2, true), (3, false)]) {
        Err(FolderError::AttemptsUnsaved { paths }) => {
            assert_eq!(paths, vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3")])
        }
        other => panic!("expected AttemptsUnsaved, got {other:?}"),
    }
}

#[test]
fn an_unsaved_folderless_attempt_is_refused_naming_its_branch() {
    let attempts = vec![attempt(1, A, None), attempt(2, B, Some("/t/try-2"))];
    match drop_plan(&attempts, &[], &[(2, true)]) {
        Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![PathBuf::from("refs/heads/try/z/r/1")]),
        other => panic!("expected AttemptsUnsaved, got {other:?}"),
    }
}

#[test]
fn no_attempts_plan_nothing() {
    assert_eq!(drop_plan(&[], &[], &[]).unwrap(), Dropped { remove: Vec::new(), delete: Vec::new() });
}

#[test]
fn orphans_are_refused_before_unsaved_attempts_naming_every_orphan_folder() {
    let attempts = vec![attempt(1, A, Some("/t/try-1"))];
    let orphans = vec![(2, PathBuf::from("/t/try-2")), (3, PathBuf::from("/t/try-3"))];
    match drop_plan(&attempts, &orphans, &[(3, true), (2, false)]) {
        Err(FolderError::OrphanAttempt { paths }) => {
            assert_eq!(paths, vec![PathBuf::from("/t/try-2"), PathBuf::from("/t/try-3")])
        }
        other => panic!("expected OrphanAttempt, got {other:?}"),
    }
}

#[test]
fn a_saved_orphan_is_still_refused() {
    let attempts = vec![attempt(3, C, Some("/t/try-3"))];
    let orphans = vec![(1, PathBuf::from("/t/try-1"))];
    match drop_plan(&attempts, &orphans, &[(1, true), (3, true)]) {
        Err(FolderError::OrphanAttempt { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-1")]),
        other => panic!("expected OrphanAttempt, got {other:?}"),
    }
}

#[test]
fn orphans_alone_are_refused_in_k_order() {
    let orphans = vec![(4, PathBuf::from("/t/try-4")), (2, PathBuf::from("/t/try-2"))];
    match drop_plan(&[], &orphans, &[(2, true), (4, true)]) {
        Err(FolderError::OrphanAttempt { paths }) => {
            assert_eq!(paths, vec![PathBuf::from("/t/try-2"), PathBuf::from("/t/try-4")])
        }
        other => panic!("expected OrphanAttempt, got {other:?}"),
    }
}
