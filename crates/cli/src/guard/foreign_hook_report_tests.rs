//! foreign_hook_report: a line per foreign hook, the repository's hooks as owner, the first path in the next.

use std::path::PathBuf;

use super::foreign_hook_report;
use crate::report::{ExitCode, Report};

#[test]
fn one_foreign_hook() {
    assert_eq!(
        foreign_hook_report(&[PathBuf::from("/r/.git/hooks/pre-push")]),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Refused,
            outcome: "foreign-hook".to_string(),
            lines: vec![
                "/r/.git/hooks/pre-push is a hook Mortar did not write; Mortar never overwrites one".to_string()
            ],
            owner: Some("the repository's own hooks".to_string()),
            next: "move /r/.git/hooks/pre-push aside or fold it into your own hook by hand, then mortar guard install \
                   --actor <name>"
                .to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn two_foreign_hooks_each_get_a_line_and_the_next_names_the_first() {
    let report = foreign_hook_report(&[
        PathBuf::from("/r/hooks/reference-transaction"),
        PathBuf::from("/r/hooks/pre-push"),
    ]);
    assert_eq!(
        report.lines,
        vec![
            "/r/hooks/reference-transaction is a hook Mortar did not write; Mortar never overwrites one".to_string(),
            "/r/hooks/pre-push is a hook Mortar did not write; Mortar never overwrites one".to_string(),
        ]
    );
    assert_eq!(
        report.next,
        "move /r/hooks/reference-transaction aside or fold it into your own hook by hand, then mortar guard install \
         --actor <name>"
    );
}
