use crate::PathKind;
use std::collections::HashSet;

#[test]
fn the_three_kinds_are_distinct() {
    let kinds: HashSet<PathKind> = [PathKind::File, PathKind::Folder, PathKind::Gitlink].into_iter().collect();
    assert_eq!(kinds.len(), 3);
}

#[test]
fn a_copy_leaves_the_original_usable_and_equal() {
    let kind = PathKind::Folder;
    let copy = kind;
    assert_eq!(kind, copy);
    assert_ne!(copy, PathKind::File);
}
