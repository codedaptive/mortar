//! with_moved_first: the moved line, when given, comes first and the other lines keep their order. Unit tests: pure.

use super::with_moved_first;

#[test]
fn none_leaves_the_lines_unchanged() {
    assert_eq!(
        with_moved_first(None, vec!["join moved no ref".to_string(), "the file is kept".to_string()]),
        vec!["join moved no ref".to_string(), "the file is kept".to_string()]
    );
}

#[test]
fn some_puts_the_line_first() {
    assert_eq!(
        with_moved_first(
            Some("the snapshot ref moved as the work was saved first"),
            vec!["join moved no ref".to_string(), "the file is kept".to_string()],
        ),
        vec![
            "the snapshot ref moved as the work was saved first".to_string(),
            "join moved no ref".to_string(),
            "the file is kept".to_string(),
        ]
    );
}

#[test]
fn empty_lines_with_some_give_just_the_moved_line() {
    assert_eq!(
        with_moved_first(Some("the snapshot ref moved as the work was saved first"), Vec::new()),
        vec!["the snapshot ref moved as the work was saved first".to_string()]
    );
}
