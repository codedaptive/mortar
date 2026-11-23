//! default_commit_message: "commit-files: " and the paths joined by ", ".

use std::path::PathBuf;

use super::default_commit_message;

#[test]
fn the_message_names_the_paths_joined_by_commas() {
    for (paths, expected) in [
        (&["src/a.rs"][..], "commit-files: src/a.rs"),
        (&["src/a.rs", "docs/b c.md", "x"], "commit-files: src/a.rs, docs/b c.md, x"),
    ] {
        let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        assert_eq!(default_commit_message(&paths), expected, "{paths:?}");
    }
}
