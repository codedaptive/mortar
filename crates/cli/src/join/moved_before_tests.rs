use super::moved_before;

#[test]
fn neither_ref_moved() {
    assert_eq!(moved_before(false, false, "refs/heads/req/core/fix-1"), None);
}

#[test]
fn only_the_request_branch_moved() {
    assert_eq!(
        moved_before(true, false, "refs/heads/req/core/fix-1"),
        Some("refs/heads/req/core/fix-1 moved as the work was saved first".to_string())
    );
}

#[test]
fn only_the_snapshot_ref_moved() {
    assert_eq!(
        moved_before(false, true, "refs/heads/req/core/fix-1"),
        Some("the snapshot ref moved as the work was saved first".to_string())
    );
}

#[test]
fn both_refs_moved() {
    assert_eq!(
        moved_before(true, true, "refs/heads/req/core/fix-1"),
        Some("refs/heads/req/core/fix-1 and the snapshot ref moved as the work was saved first".to_string())
    );
}
