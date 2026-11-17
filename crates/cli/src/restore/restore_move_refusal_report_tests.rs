//! restore_move_refusal_report: the refs are restored, the folders already moved are named, then the folder that
//! could not move and why; a plain repeat finishes the move (S-113(3)).

use std::path::{Path, PathBuf};

use super::restore_move_refusal_report;
use crate::report::{ExitCode, Report};

#[test]
fn the_report_says_the_refs_moved_and_names_the_folder_that_did_not() {
    assert_eq!(
        restore_move_refusal_report(
            "op-1",
            &[PathBuf::from("/w/a")],
            Path::new("/w/b"),
            "refs/heads/req/core/b",
            "files stand in the way"
        ),
        Report {
            verb: "restore".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "the refs are restored as of op op-1".to_string(),
                "moved the folder /w/a to its ref's restored value".to_string(),
                "the folder /w/b on refs/heads/req/core/b could not move: files stand in the way".to_string(),
            ],
            owner: Some("the agent working in /w/b".to_string()),
            next: "clear what stops the folder's move, then repeat the same mr restore command; the repeat \
                   finishes the move"
                .to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn with_no_folder_moved_yet_only_the_refs_and_the_folder_are_named() {
    let report = restore_move_refusal_report("op-2", &[], Path::new("/w/b"), "refs/heads/try/core/b/1", "why");
    assert_eq!(
        report.lines,
        vec![
            "the refs are restored as of op op-2".to_string(),
            "the folder /w/b on refs/heads/try/core/b/1 could not move: why".to_string(),
        ]
    );
}
