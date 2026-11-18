//! resolve_refusal_report: no paused join is exit 0; markers left are exit 2 with each file's marker lines. The
//! refusals whose lines are join's own texts (a stale state, a conflicted path that is a folder or refused, a corrupt
//! state and the rest under join_refusal_report) are ZITs in crates/cli/tests/resolve_refusal_report_tests.rs.
//! ZUT (rule 11): both cases return before join_refusal_report; the errors are built as data.

use std::path::PathBuf;

use ::join::{JoinRunError, MarkedFile};

use super::resolve_refusal_report;
use crate::report::{ExitCode, Report};

const STATE: &str = "/r/.git/worktrees/fix-1/mortar-join.json";

#[test]
fn no_paused_join_is_nothing_to_resolve() {
    assert_eq!(
        resolve_refusal_report(&JoinRunError::NothingToResolve { state: PathBuf::from(STATE) }),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-resolve".to_string(),
            lines: vec!["no join is paused in this folder; nothing to resolve".to_string()],
            owner: None,
            next: "mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn markers_left_name_each_file_and_its_lines() {
    let files = vec![
        MarkedFile { path: PathBuf::from("src/a.rs"), lines: vec![3, 7, 12] },
        MarkedFile { path: PathBuf::from("b.txt"), lines: vec![1] },
    ];
    assert_eq!(
        resolve_refusal_report(&JoinRunError::StillMarked(files)),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Conflict,
            outcome: "still-marked".to_string(),
            lines: vec![
                "conflict markers remain in these files; no commit was made and no ref moved:".to_string(),
                "  src/a.rs (lines 3, 7, 12)".to_string(),
                "  b.txt (lines 1)".to_string(),
            ],
            owner: None,
            next: "fix the files, then mr resolve".to_string(),
            fields: Vec::new(),
        }
    );
}
