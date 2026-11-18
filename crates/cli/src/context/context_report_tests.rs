//! context_report: a worktree without a record is refused "not-a-folder" naming it; a bad record is refused naming
//! the file. The cases whose lines carry gitio's or folders' own text (outside a repository, an unreadable record,
//! another folder error, git failing) are ZITs in crates/cli/tests/context_report_tests.rs.
//! ZUT (rule 11): the error values are built as data; neither case reaches another file's code.

use std::path::PathBuf;

use folders::record::RecordError;
use folders::FolderError;

use super::{context_report, ContextError};
use crate::report::ExitCode;

#[test]
fn a_worktree_without_a_record_is_not_a_folder_naming_it() {
    let report = context_report("join", &ContextError::NotAFolder { toplevel: PathBuf::from("/r/main") });
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "not-a-folder");
    assert_eq!(
        report.lines,
        vec!["this directory is not in a Mortar folder: the worktree /r/main has no Mortar record".to_string()]
    );
    assert_eq!(report.owner.as_deref(), Some("the harness that makes Mortar folders"));
    assert_eq!(report.next, "run mr join inside a folder Mortar made");
}

#[test]
fn a_record_mortar_did_not_write_is_refused_naming_the_file() {
    let error = ContextError::Record(FolderError::Record {
        path: PathBuf::from("/r/.git/worktrees/a/mortar-folder.json"),
        error: RecordError { reason: "zone is missing".to_string() },
    });
    let report = context_report("save", &error);
    assert_eq!(report.verb, "save");
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "refused");
    assert_eq!(
        report.lines[0],
        "the folder record /r/.git/worktrees/a/mortar-folder.json is not one Mortar writes: zone is missing"
    );
    assert_eq!(report.owner.as_deref(), Some("the harness that makes Mortar folders"));
}
