//! commit_files_report: a commit is "committed" naming the ref's move, path or paths by count; none is "same" with
//! nothing written; both carry the fields commit, ref, moved and paths.

use std::path::PathBuf;

use gitio::Oid;

use super::commit_files_report;
use crate::report::{ExitCode, OutValue, Report};

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn report(outcome: &str, lines: &[&str], commit: &str, moved: bool, paths: &[&str]) -> Report {
    Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: None,
        next: "the next step".to_string(),
        fields: vec![
            ("commit".to_string(), OutValue::Text(commit.to_string())),
            ("ref".to_string(), OutValue::Text("refs/heads/zone/core".to_string())),
            ("moved".to_string(), OutValue::Bool(moved)),
            ("paths".to_string(), OutValue::Texts(paths.iter().map(|path| path.to_string()).collect())),
        ],
    }
}

#[test]
fn committed_and_same_are_reported_with_their_fields() {
    let new = oid(NEW);
    let left = "the folder's other changes were left as they are";
    for (commit, paths, expected) in [
        (
            Some(&new),
            &["src/a.rs"][..],
            report(
                "committed",
                &[
                    "committed 1 named path as 2222222222222222222222222222222222222222: refs/heads/zone/core moved \
                     from 1111111111111111111111111111111111111111 to 2222222222222222222222222222222222222222",
                    left,
                ],
                NEW,
                true,
                &["src/a.rs"],
            ),
        ),
        (
            Some(&new),
            &["src/a.rs", "b"],
            report(
                "committed",
                &[
                    "committed 2 named paths as 2222222222222222222222222222222222222222: refs/heads/zone/core moved \
                     from 1111111111111111111111111111111111111111 to 2222222222222222222222222222222222222222",
                    left,
                ],
                NEW,
                true,
                &["src/a.rs", "b"],
            ),
        ),
        (
            None,
            &["src/a.rs"],
            report(
                "same",
                &["refs/heads/zone/core at 1111111111111111111111111111111111111111 already holds the named paths \
                   as the folder has them; nothing was written and no ref moved"],
                OLD,
                false,
                &["src/a.rs"],
            ),
        ),
    ] {
        let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let actual =
            commit_files_report("refs/heads/zone/core", &oid(OLD), commit, &paths, "the next step".to_string());
        assert_eq!(actual, expected, "{commit:?} {paths:?}");
    }
}
