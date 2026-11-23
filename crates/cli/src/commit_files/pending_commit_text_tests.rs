//! pending_commit_text: the tip on line 1, then one path per line, every line ended by "\n".

use std::path::PathBuf;

use gitio::Oid;

use super::{pending_commit_text, PendingCommit};

const TIP: &str = "1111111111111111111111111111111111111111";

fn pending(paths: &[&str]) -> PendingCommit {
    PendingCommit { tip: Oid::parse(TIP).unwrap(), paths: paths.iter().map(PathBuf::from).collect() }
}

#[test]
fn the_text_is_the_tip_then_each_path_on_its_own_line() {
    for (paths, expected) in [
        (&["src/a.rs"][..], "1111111111111111111111111111111111111111\nsrc/a.rs\n"),
        (&["src/a.rs", "docs/b c.md"], "1111111111111111111111111111111111111111\nsrc/a.rs\ndocs/b c.md\n"),
    ] {
        assert_eq!(pending_commit_text(&pending(paths)), expected, "{paths:?}");
    }
}
