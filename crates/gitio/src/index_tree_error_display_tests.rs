use crate::IndexTreeError;
use std::path::PathBuf;

fn unmerged(paths: &[&str]) -> String {
    IndexTreeError::Unmerged { paths: paths.iter().map(PathBuf::from).collect() }.to_string()
}

#[test]
fn one_path_is_named_quoted() {
    assert_eq!(unmerged(&["top.txt"]), "index has unmerged paths: \"top.txt\"");
}

#[test]
fn several_paths_are_named_in_order_joined_by_commas() {
    assert_eq!(
        unmerged(&["sp ace", "sub/inner.txt", "top.txt"]),
        "index has unmerged paths: \"sp ace\", \"sub/inner.txt\", \"top.txt\""
    );
}

#[test]
fn a_newline_and_a_quote_in_a_path_show_escaped() {
    assert_eq!(unmerged(&["nl\nname", "a\"b"]), "index has unmerged paths: \"nl\\nname\", \"a\\\"b\"");
}
