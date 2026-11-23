//! commit_files_error_report: NotLanding, NotLanded and FolderOffTip refused in the cli's own words, owned by
//! integration; Paths through commit_paths_error_report and Land through land_error_report, under commit-files.

use std::path::PathBuf;

use gitio::{GitError, Oid};
use land::{CommitFilesError, LandError};
use snapshot::CommitPathsError;

use super::commit_files_error_report;
use crate::report::{ExitCode, Report};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const OFF_TIP_NEXT: &str = "repeat the landing verb that last moved integration (mr land, mr back-out or mr publish \
                            finish a stopped integration folder move), then mr commit-files again";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn report(exit: ExitCode, outcome: &str, lines: &[&str], owner: Option<&str>, next: &str) -> Report {
    Report {
        verb: "commit-files".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: owner.map(str::to_string),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn refused(line: &str, next: &str) -> Report {
    let lines = [line, "nothing was written and no ref moved"];
    report(ExitCode::Refused, "refused", &lines, Some("refs/heads/integ"), next)
}

#[test]
fn each_error_is_reported_in_the_clis_words() {
    for (error, expected) in [
        (
            CommitFilesError::NotLanding,
            refused(
                "no landing is in flight on integ: nothing has landed since the last publish",
                "mr land <commit> --actor <name> lands a squash; then commit-files adds to that landing",
            ),
        ),
        (
            CommitFilesError::NotLanded { landed: oid(A), integration: oid(B) },
            refused(
                "the landing state names 1111111111111111111111111111111111111111, but integration is at \
                 2222222222222222222222222222222222222222",
                "find the op that moved it with mr log --ref refs/heads/integ, then mr restore <op id> \
                 refs/heads/integ --actor <name>",
            ),
        ),
        (
            CommitFilesError::FolderOffTip { folder: Some(oid(A)), integration: oid(B) },
            refused(
                "the integration folder stands on 1111111111111111111111111111111111111111, not on integration's \
                 tip 2222222222222222222222222222222222222222",
                OFF_TIP_NEXT,
            ),
        ),
        (
            CommitFilesError::FolderOffTip { folder: None, integration: oid(B) },
            refused(
                "the integration folder stands on no commit, not on integration's tip \
                 2222222222222222222222222222222222222222",
                OFF_TIP_NEXT,
            ),
        ),
        (
            CommitFilesError::Paths(CommitPathsError::Absolute { path: PathBuf::from("/x") }),
            report(
                ExitCode::Refused,
                "refused",
                &["/x is absolute; paths are named relative to the folder", "nothing was written and no ref moved"],
                None,
                "name paths inside the folder, relative to its top",
            ),
        ),
        (
            CommitFilesError::Land(LandError::Git(GitError::refusal(Vec::new(), "gitio text"))),
            report(
                ExitCode::Failure,
                "failed",
                &["git failed while reading or moving integration"],
                None,
                "repeat the same command",
            ),
        ),
    ] {
        assert_eq!(commit_files_error_report(&error, "integ"), expected, "{error:?}");
    }
}
