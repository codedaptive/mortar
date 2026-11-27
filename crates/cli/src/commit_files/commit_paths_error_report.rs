//! The report of snapshot's commit_paths refusing or failing, in the cli's own words: snapshot's and gitio's Display
//! texts are never printed, only the paths and commit an error carries. commit_paths_error_report_tests covers it.

use gitio::TreeWithPathsRefusal;
use snapshot::CommitPathsError;

use crate::report::{failure_report, ExitCode, Report};

const INSIDE: &str = "name paths inside the folder, relative to its top";
const FILES: &str = "name the files inside the folder, not the folder that holds them";

/// - Absolute, ParentPart, ThroughSymlink: refused, naming the path (and the symlinked folder); next names paths
///   inside the folder.
/// - NotAdded: refused, git ignores the path; next names paths git tracks or does not ignore.
/// - Paths: refused, naming the path or paths gitio's refusal carries (paths_line).
/// - Ancestor, Git: failure_report, git or the file system failed while reading the paths.
/// - IndexNotSet: failure_report naming the commit written, with the index not set and no ref moved.
///
/// Every refusal is exit 3, outcome "refused", no owner and no fields, its last line "nothing was written and no
/// ref moved".
pub fn commit_paths_error_report(verb: &str, error: &CommitPathsError) -> Report {
    let (line, next) = match error {
        CommitPathsError::Absolute { path } => {
            (format!("{} is absolute; paths are named relative to the folder", path.display()), INSIDE)
        }
        CommitPathsError::ParentPart { path } => {
            (format!("{} has a .. part, which could leave the folder", path.display()), INSIDE)
        }
        CommitPathsError::ThroughSymlink { path, link } => (
            format!("{} is reached through the symlinked folder {}", path.display(), link.display()),
            INSIDE,
        ),
        CommitPathsError::NotAdded { path } => (
            format!("git ignores {}, so it cannot be committed", path.display()),
            "name paths git tracks or does not ignore",
        ),
        CommitPathsError::Paths(refusal) => paths_line(refusal),
        CommitPathsError::Ancestor { .. } | CommitPathsError::Git(_) => {
            return failure_report(verb, "git or the file system failed while reading the named paths");
        }
        CommitPathsError::IndexNotSet { commit, .. } => {
            let what = format!(
                "commit {commit} was written but the folder's index was not set for the named paths; no ref moved"
            );
            return failure_report(verb, &what);
        }
    };
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line, "nothing was written and no ref moved".to_string()],
        owner: None,
        next: next.to_string(),
        fields: Vec::new(),
    }
}

/// The line and next for gitio's refusal of the path list. Its `from` is the folder as git hashes it and its `base`
/// the tree of the commit the new one builds on. An Unusable path is shown quoted, so an empty one is seen.
fn paths_line(refusal: &TreeWithPathsRefusal) -> (String, &'static str) {
    match refusal {
        TreeWithPathsRefusal::FolderInFrom { path } => (format!("{} is a folder in the folder", path.display()), FILES),
        TreeWithPathsRefusal::FolderInBase { path } => (
            format!("{} is gone from the folder and is a folder in the commit it builds on", path.display()),
            FILES,
        ),
        TreeWithPathsRefusal::Overlap { folder, inner } => (
            format!("{} is named, and so is {} inside it", folder.display(), inner.display()),
            FILES,
        ),
        TreeWithPathsRefusal::NotAsStored { path } => (
            format!(
                "{} is not written as git stores it: it has a leading or trailing /, or an empty, . or .. part",
                path.display()
            ),
            INSIDE,
        ),
        TreeWithPathsRefusal::Unusable { path } => (format!("the path {path:?} is empty or holds a NUL"), INSIDE),
    }
}
