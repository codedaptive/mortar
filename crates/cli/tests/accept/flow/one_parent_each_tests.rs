use super::one_parent_each;

#[test]
fn commits_with_one_parent_each_are_ordinary() {
    assert!(one_parent_each("c3 c2\nc2 c1\n"));
}

#[test]
fn a_merge_is_not() {
    assert!(!one_parent_each("c3 c2 c9\nc2 c1\n"));
}

#[test]
fn a_root_is_not() {
    assert!(!one_parent_each("c1\n"));
}

#[test]
fn no_commits_are_not() {
    assert!(!one_parent_each(""));
    assert!(!one_parent_each("\n"));
}
