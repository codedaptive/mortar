//! same_staged_report: "same" with the ref unmoved, a second line and a field staged naming the paths a stopped run
//! left staged, and next git restore --staged of those paths.

use std::path::PathBuf;

use gitio::Oid;

use super::same_staged_report;
use crate::report::{ExitCode, OutValue, Report};

const OLD: &str = "1111111111111111111111111111111111111111";

#[test]
fn same_names_the_staged_paths_and_restores_them_next() {
    let same = "refs/heads/zone/core at 1111111111111111111111111111111111111111 already holds the named paths as the \
                folder has them; nothing was written and no ref moved";
    let fields = |paths: &[&str]| {
        let texts: Vec<String> = paths.iter().map(|path| path.to_string()).collect();
        vec![
            ("commit".to_string(), OutValue::Text(OLD.to_string())),
            ("ref".to_string(), OutValue::Text("refs/heads/zone/core".to_string())),
            ("moved".to_string(), OutValue::Bool(false)),
            ("paths".to_string(), OutValue::Texts(texts.clone())),
            ("staged".to_string(), OutValue::Texts(texts)),
        ]
    };
    for (paths, expected) in [
        (
            &["src/a.rs"][..],
            Report {
                verb: "commit-files".to_string(),
                exit: ExitCode::Done,
                outcome: "same".to_string(),
                lines: vec![
                    same.to_string(),
                    "a stopped mr commit-files left the named paths staged in the folder's index: src/a.rs".to_string(),
                ],
                owner: None,
                next: "git restore --staged -- src/a.rs".to_string(),
                fields: fields(&["src/a.rs"]),
            },
        ),
        (
            &["src/a.rs", "b"],
            Report {
                verb: "commit-files".to_string(),
                exit: ExitCode::Done,
                outcome: "same".to_string(),
                lines: vec![
                    same.to_string(),
                    "a stopped mr commit-files left the named paths staged in the folder's index: src/a.rs, b"
                        .to_string(),
                ],
                owner: None,
                next: "git restore --staged -- src/a.rs b".to_string(),
                fields: fields(&["src/a.rs", "b"]),
            },
        ),
    ] {
        let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let actual = same_staged_report("refs/heads/zone/core", &Oid::parse(OLD).unwrap(), &paths);
        assert_eq!(actual, expected, "{paths:?}");
    }
}
