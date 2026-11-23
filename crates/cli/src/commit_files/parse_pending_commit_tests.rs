//! parse_pending_commit: a tip line and at least one path line, the final "\n" optional, is read; pending_commit_text
//! reads back to the value it was made from; an empty text, a bad tip, no path line or an empty path line is None.

use std::path::PathBuf;

use gitio::Oid;

use super::{parse_pending_commit, pending_commit_text, PendingCommit};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";

fn pending(tip: &str, paths: &[&str]) -> PendingCommit {
    PendingCommit { tip: Oid::parse(tip).unwrap(), paths: paths.iter().map(PathBuf::from).collect() }
}

#[test]
fn a_marker_is_read_or_refused() {
    for (text, expected) in [
        ("1111111111111111111111111111111111111111\nsrc/a.rs\n", Some(pending(A, &["src/a.rs"]))),
        ("1111111111111111111111111111111111111111\nsrc/a.rs", Some(pending(A, &["src/a.rs"]))),
        ("2222222222222222222222222222222222222222\na\nb c/d\n", Some(pending(B, &["a", "b c/d"]))),
        ("", None),
        ("\n", None),
        ("1111111111111111111111111111111111111111", None),
        ("1111111111111111111111111111111111111111\n", None),
        ("1111111111111111111111111111111111111111\n\n", None),
        ("1111111111111111111111111111111111111111\na\n\nb\n", None),
        ("1111111111111111111111111111111111111111\na\n\n", None),
        ("not an oid\nsrc/a.rs\n", None),
        ("111111111111111111111111111111111111111\nsrc/a.rs\n", None),
        ("\n1111111111111111111111111111111111111111\nsrc/a.rs\n", None),
    ] {
        assert_eq!(parse_pending_commit(text), expected, "{text:?}");
    }
}

#[test]
fn the_text_of_a_marker_reads_back_to_it() {
    for value in [
        pending(A, &["src/a.rs"]),
        pending(B, &["a", "b/c", "d e.md"]),
        pending(A, &[".hidden", "x/y/z.toml", "-dash"]),
    ] {
        assert_eq!(parse_pending_commit(&pending_commit_text(&value)), Some(value.clone()), "{value:?}");
    }
}
