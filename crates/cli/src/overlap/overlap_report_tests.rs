//! overlap_report: exit 0 either way; "no-overlap" when no two open requests of the zone change a path, else
//! "overlaps" with one line per pair naming both requests and the paths both change.

use std::path::PathBuf;

use ::join::OverlapPair;

use super::overlap_report;
use crate::report::{ExitCode, OutValue, Report};

fn pair(first: &str, second: &str, paths: &[&str]) -> OverlapPair {
    OverlapPair {
        first: first.to_string(),
        second: second.to_string(),
        paths: paths.iter().map(PathBuf::from).collect(),
    }
}

#[test]
fn no_pairs_is_no_overlap() {
    assert_eq!(
        overlap_report("core", &[]),
        Report {
            verb: "overlap".to_string(),
            exit: ExitCode::Done,
            outcome: "no-overlap".to_string(),
            lines: vec!["no two open requests of zone core change the same path".to_string()],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![("pairs".to_string(), OutValue::Texts(Vec::new()))],
        }
    );
}

#[test]
fn two_pairs_give_a_line_each_with_the_paths() {
    let pairs = [
        pair("fix-1", "fix-2", &["src/a.rs", "src/b.rs"]),
        pair("fix-1", "fix-3", &["README.md"]),
    ];
    assert_eq!(
        overlap_report("core", &pairs),
        Report {
            verb: "overlap".to_string(),
            exit: ExitCode::Done,
            outcome: "overlaps".to_string(),
            lines: vec![
                "requests fix-1 and fix-2 both change: src/a.rs, src/b.rs".to_string(),
                "requests fix-1 and fix-3 both change: README.md".to_string(),
            ],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![(
                "pairs".to_string(),
                OutValue::Texts(vec!["fix-1 fix-2".to_string(), "fix-1 fix-3".to_string()])
            )],
        }
    );
}
