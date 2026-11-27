//! The report of land's commit_files refusing or failing, in the cli's own words: land's, snapshot's, gitio's and
//! oplog's Display texts are never printed, only the oids an error carries. commit_files_error_report_tests covers it.

use land::CommitFilesError;

use super::commit_paths_error_report;
use crate::landing::{land_error_report, landing_refusal};
use crate::report::Report;

const VERB: &str = "commit-files";

/// - NotLanding: refused, no landing is in flight; next lands a squash first.
/// - NotLanded: refused, the landing state and integration disagree; next finds the op that moved integration in the
///   log and restores it, as land_error_report's IntegrationElsewhere does.
/// - FolderOffTip: refused, the integration folder is not on integration's tip; next repeats the landing verb that
///   last moved integration.
/// - Paths: commit_paths_error_report; Land: land_error_report.
///
/// Each NotLanding, NotLanded and FolderOffTip refusal is exit 3, outcome "refused", owner
/// refs/heads/<integration_branch>, no fields, its last line "nothing was written and no ref moved".
pub fn commit_files_error_report(error: &CommitFilesError, integration_branch: &str) -> Report {
    let integration_ref = format!("refs/heads/{integration_branch}");
    let (line, next) = match error {
        CommitFilesError::NotLanding => (
            format!("no landing is in flight on {integration_branch}: nothing has landed since the last publish"),
            "mr land <commit> --actor <name> lands a squash; then commit-files adds to that landing".to_string(),
        ),
        CommitFilesError::NotLanded { landed, integration } => (
            format!("the landing state names {landed}, but integration is at {integration}"),
            format!(
                "find the op that moved it with mr log --ref {integration_ref}, then mr restore <op id> \
                 {integration_ref} --actor <name>"
            ),
        ),
        CommitFilesError::FolderOffTip { folder, integration } => {
            let folder = folder.as_ref().map_or("no commit".to_string(), ToString::to_string);
            (
                format!("the integration folder stands on {folder}, not on integration's tip {integration}"),
                "repeat the landing verb that last moved integration (mr land, mr back-out or mr publish finish a \
                 stopped integration folder move), then mr commit-files again"
                    .to_string(),
            )
        }
        CommitFilesError::Paths(error) => return commit_paths_error_report(VERB, error),
        CommitFilesError::Land(error) => return land_error_report(VERB, error, integration_branch),
    };
    landing_refusal(VERB, vec![line, "nothing was written and no ref moved".to_string()], &integration_ref, next)
}
