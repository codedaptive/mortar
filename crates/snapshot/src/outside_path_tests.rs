use crate::outside_path::outside_path;
use crate::CommitPathsError;
use std::path::{Path, PathBuf};

/// The refusal's variant name and the path it carries, or None.
fn refusal(path: &str) -> Option<(&'static str, PathBuf)> {
    match outside_path(Path::new(path)) {
        None => None,
        Some(CommitPathsError::Absolute { path }) => Some(("Absolute", path)),
        Some(CommitPathsError::ParentPart { path }) => Some(("ParentPart", path)),
        Some(other) => panic!("outside_path gave {other:?}"),
    }
}

#[test]
fn outside_path_refuses_a_root_or_a_parent_part_and_passes_the_rest() {
    let rows: [(&str, Option<&str>); 11] = [
        ("/abs/x", Some("Absolute")),
        ("/a/../b", Some("Absolute")),
        ("../x", Some("ParentPart")),
        ("a/../b", Some("ParentPart")),
        ("a/..", Some("ParentPart")),
        ("..", Some("ParentPart")),
        ("a", None),
        ("a/b/c.txt", None),
        ("./a", None),
        ("a..b/..c", None),
        ("...", None),
    ];
    for (path, expected) in rows {
        let want = expected.map(|kind| (kind, PathBuf::from(path)));
        assert_eq!(refusal(path), want, "path {path:?}");
    }
}
