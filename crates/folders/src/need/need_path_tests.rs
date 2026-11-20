//! need_path: a clean path back as it is or with its trailing '/' removed; a line break and sparse_paths'
//! refusals refused.

use super::need_path::need_path;

#[test]
fn a_clean_path_comes_back_as_is() {
    assert_eq!(need_path("crates/gitio").unwrap(), "crates/gitio");
}

#[test]
fn a_trailing_slash_is_removed() {
    assert_eq!(need_path("crates/gitio/").unwrap(), "crates/gitio");
}

#[test]
fn a_line_break_is_refused() {
    assert_eq!(need_path("crates/a\nb").unwrap_err().reason, "sparse path \"crates/a\\nb\" holds a line break");
}

#[test]
fn a_sparse_paths_refusal_passes_through() {
    assert_eq!(
        need_path("../x").unwrap_err().reason,
        "sparse path \"../x\" has a \".\", \"..\" or empty component"
    );
}
