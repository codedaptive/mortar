use crate::save_head::save_head;
use crate::SnapshotError;
use gitio::Oid;
use std::path::{Path, PathBuf};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn branch_and_commit_are_returned() {
    let head = (Some("refs/heads/zone/snapshot".to_string()), Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")));
    assert_eq!(
        save_head(Path::new("/w/f"), head).unwrap(),
        ("refs/heads/zone/snapshot".to_string(), oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
    );
}

#[test]
fn unborn_on_a_branch_is_unborn_head() {
    match save_head(Path::new("/w/f"), (Some("refs/heads/main".to_string()), None)) {
        Err(SnapshotError::UnbornHead { folder }) => assert_eq!(folder, PathBuf::from("/w/f")),
        other => panic!("expected UnbornHead, got {other:?}"),
    }
}

#[test]
fn detached_is_detached_head() {
    match save_head(Path::new("/w/f"), (None, Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")))) {
        Err(SnapshotError::DetachedHead { folder }) => assert_eq!(folder, PathBuf::from("/w/f")),
        other => panic!("expected DetachedHead, got {other:?}"),
    }
}

#[test]
fn no_branch_and_no_commit_is_unborn_head() {
    match save_head(Path::new("/w/f"), (None, None)) {
        Err(SnapshotError::UnbornHead { folder }) => assert_eq!(folder, PathBuf::from("/w/f")),
        other => panic!("expected UnbornHead, got {other:?}"),
    }
}
