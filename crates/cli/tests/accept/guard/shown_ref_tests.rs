use super::shown_ref;

#[test]
fn an_empty_read_is_absent() {
    assert_eq!(shown_ref(""), "absent");
}

#[test]
fn a_commit_is_shown_as_it_is() {
    assert_eq!(shown_ref("d19b4388d79f56df3bd0037ba2cc8c0b446ea82c"), "d19b4388d79f56df3bd0037ba2cc8c0b446ea82c");
}
