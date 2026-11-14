//! unsaved_attempts: unlisted and false entries named in k order, by folder or else by full branch name; the
//! excepted attempt never named.

use std::path::PathBuf;

use gitio::Oid;

use super::unsaved_attempts;
use super::Attempt;

fn attempt(k: u32, folder: Option<&str>) -> Attempt {
    Attempt {
        k,
        branch: format!("refs/heads/try/z/r/{k}"),
        tip: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
        folder: folder.map(PathBuf::from),
    }
}

#[test]
fn every_saved_attempt_leaves_nothing() {
    let attempts = vec![attempt(1, Some("/t/try-1")), attempt(2, None)];
    assert_eq!(unsaved_attempts(&attempts, &[(2, true), (1, true)], None), Vec::<PathBuf>::new());
}

#[test]
fn unlisted_and_false_attempts_are_named_in_k_order() {
    let attempts = vec![attempt(3, Some("/t/try-3")), attempt(1, Some("/t/try-1")), attempt(2, Some("/t/try-2"))];
    assert_eq!(
        unsaved_attempts(&attempts, &[(2, true), (3, false)], None),
        vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3")]
    );
}

#[test]
fn a_folderless_attempt_is_named_by_its_full_branch() {
    let attempts = vec![attempt(1, None), attempt(2, Some("/t/try-2"))];
    assert_eq!(
        unsaved_attempts(&attempts, &[], None),
        vec![PathBuf::from("refs/heads/try/z/r/1"), PathBuf::from("/t/try-2")]
    );
}

#[test]
fn the_excepted_attempt_is_never_named() {
    let attempts = vec![attempt(1, Some("/t/try-1")), attempt(2, Some("/t/try-2"))];
    assert_eq!(unsaved_attempts(&attempts, &[(2, false)], Some(2)), vec![PathBuf::from("/t/try-1")]);
}

#[test]
fn an_entry_for_another_attempt_does_not_save_this_one() {
    let attempts = vec![attempt(1, Some("/t/try-1"))];
    assert_eq!(unsaved_attempts(&attempts, &[(2, true)], None), vec![PathBuf::from("/t/try-1")]);
}

#[test]
fn no_attempts_leave_nothing() {
    assert_eq!(unsaved_attempts(&[], &[], None), Vec::<PathBuf>::new());
}
