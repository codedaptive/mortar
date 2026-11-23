//! moved_since_read_refusal: refused naming the tip the run read and the paths left staged, owned by the zone; next
//! restores the paths from that tip, runs mr folder refresh (moving a named file aside if the refresh refuses on
//! it), then runs mr commit-files again.

use std::path::{Path, PathBuf};

use gitio::Oid;

use super::moved_since_read_refusal;
use crate::report::{ExitCode, Report};

const TIP: &str = "3333333333333333333333333333333333333333";

#[test]
fn the_refusal_names_the_tip_read_and_the_restore_from_it() {
    for (paths, next) in [
        (
            &["src/a.rs"][..],
            "git restore --staged --source=3333333333333333333333333333333333333333 -- src/a.rs, then mr folder \
             refresh /w/core --actor <name> (if the refresh refuses on a path, move that file aside and put it \
             back after), then mr commit-files again",
        ),
        (
            &["src/a.rs", "b/c"],
            "git restore --staged --source=3333333333333333333333333333333333333333 -- src/a.rs b/c, then mr folder \
             refresh /w/core --actor <name> (if the refresh refuses on a path, move that file aside and put it \
             back after), then mr commit-files again",
        ),
    ] {
        let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let expected = Report {
            verb: "commit-files".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "zone/core moved since it was read at 3333333333333333333333333333333333333333, so no ref moved"
                    .to_string(),
                "the named paths were left staged as committed onto 3333333333333333333333333333333333333333"
                    .to_string(),
            ],
            owner: Some("zone core".to_string()),
            next: next.to_string(),
            fields: Vec::new(),
        };
        let actual = moved_since_read_refusal("core", Path::new("/w/core"), &Oid::parse(TIP).unwrap(), &paths);
        assert_eq!(actual, expected, "{paths:?}");
    }
}
