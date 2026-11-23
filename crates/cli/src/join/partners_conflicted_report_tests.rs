//! partners_conflicted_report: a conflicting new merge as its exact refusal, with and without a save's moved line.

use std::path::PathBuf;

use super::partners_conflicted_report;
use crate::report::{ExitCode, OutValue, Report};

fn refusal(lines: &[&str]) -> Report {
    Report {
        verb: "join".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: Some("the coordinators of zones core and web".to_string()),
        next: "repeat mr join once zones core and web agree on these files".to_string(),
        fields: vec![
            ("partner".to_string(), OutValue::Text("web".to_string())),
            ("files".to_string(), OutValue::Texts(vec!["Cargo.toml".to_string(), "src/lib.rs".to_string()])),
        ],
    }
}

#[test]
fn partners_conflicted_report_cases() {
    let files = [PathBuf::from("Cargo.toml"), PathBuf::from("src/lib.rs")];
    let cases: Vec<(&str, Option<&str>, Report)> = vec![
        (
            "nothing moved before",
            None,
            refusal(&[
                "the new merge of zone core with partner web conflicts on these files:",
                "  Cargo.toml",
                "  src/lib.rs",
                "join moved no ref and wrote no state",
            ]),
        ),
        (
            "a save moved the branch first",
            Some("refs/heads/req/core/fix-1 moved as the work was saved first"),
            refusal(&[
                "refs/heads/req/core/fix-1 moved as the work was saved first",
                "the new merge of zone core with partner web conflicts on these files:",
                "  Cargo.toml",
                "  src/lib.rs",
                "join moved no ref and wrote no state",
            ]),
        ),
    ];
    for (name, moved, expected) in cases {
        assert_eq!(partners_conflicted_report("core", "web", &files, moved), expected, "case {name}");
    }
}
