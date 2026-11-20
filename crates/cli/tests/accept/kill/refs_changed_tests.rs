use super::refs_changed;

#[test]
fn the_same_listing_changes_nothing() {
    let refs = "refs/heads/main aaa\nrefs/heads/zone/a bbb\n";
    assert_eq!(refs_changed(refs, refs), Vec::<String>::new());
}

#[test]
fn moved_added_and_removed_refs_are_named_in_order() {
    let before = "refs/heads/main aaa\nrefs/heads/zone/a bbb\nrefs/mortar/squash/a ccc\n";
    let after = "refs/heads/integration ddd\nrefs/heads/main aaa\nrefs/heads/zone/a eee\n";
    assert_eq!(refs_changed(before, after), vec!["refs/heads/integration", "refs/heads/zone/a", "refs/mortar/squash/a"]);
}

#[test]
fn blank_lines_are_not_refs() {
    assert_eq!(refs_changed("\nrefs/heads/main aaa\n\n", "refs/heads/main aaa"), Vec::<String>::new());
}
