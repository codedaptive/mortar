use crate::EntryKind;

#[test]
fn commit_is_not_an_other_named_commit() {
    assert_ne!(EntryKind::Commit, EntryKind::Other("commit".to_string()));
}

#[test]
fn tree_is_not_an_other_named_tree() {
    assert_ne!(EntryKind::Tree, EntryKind::Other("tree".to_string()));
}

#[test]
fn others_are_equal_only_when_their_names_match() {
    assert_eq!(EntryKind::Other("tag".to_string()), EntryKind::Other("tag".to_string()));
    assert_ne!(EntryKind::Other("tag".to_string()), EntryKind::Other("Tag".to_string()));
}

#[test]
fn a_clone_equals_its_original() {
    let kind = EntryKind::Other("tag".to_string());
    assert_eq!(kind.clone(), EntryKind::Other("tag".to_string()));
}
