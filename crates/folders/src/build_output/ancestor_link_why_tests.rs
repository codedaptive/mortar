//! ancestor_link_why: the refusal text for a symlinked ancestor, on each side.

use std::path::Path;

use super::ancestor_link_why;

#[test]
fn ancestor_link_why_names_the_side_the_link_and_the_dir() {
    let rows = [
        (
            "parent's",
            "build",
            "build/out",
            r#"the parent's "build", an ancestor of build dir "build/out", is a symlink; nothing copied"#,
        ),
        (
            "folder's",
            "a/b",
            "a/b/out",
            r#"the folder's "a/b", an ancestor of build dir "a/b/out", is a symlink; nothing copied"#,
        ),
    ];
    for (side, link, dir, want) in rows {
        assert_eq!(ancestor_link_why(dir, side, Path::new(link)), want, "{side}");
    }
}
