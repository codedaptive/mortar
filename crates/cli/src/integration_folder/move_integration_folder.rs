//! Moves the integration folder, the worktree on the integration branch, after a verb moved that branch.
//! No sibling test (S-104): it runs git and the op log. The RITs crates/cli/tests/cli_rit/run_back_out_tests.rs
//! and crates/cli/tests/cli_rit/run_publish_tests.rs cover it through the binary. RIT left for Nagatha (cross-zone, listed only,
//! not written): a local change in the integration folder that is in the way refuses the move, exit 1
//! "folder-not-moved".

use std::path::PathBuf;

use folders::{move_folder, FolderError};
use gitio::{Git, Oid};

use super::{folder_move_refusal_report, integration_folder};
use crate::folder::folder_error_why;
use crate::report::{ExitCode, Report};

/// `done` is the caller's line saying what already happened (such as "integration is back at <oid>"). `old` equal
/// to `new` is None with no git run. Else the folder on `integration_branch` (integration_folder): none is None; a
/// worktree list git cannot give is exit 1 "folder-not-moved", owned by the repository's worktrees, whose next is
/// the owner's step in words: find that worktree and bring its files and index to `new`. A found folder
/// is moved from `old` to `new` by folders::move_folder with `saved` true, since the integration folder is not an
/// agent's folder; move_folder's plain move still refuses over local changes in the way. A move it refuses is
/// folder_move_refusal_report with the cli's words for the error. Moved is the folder's path.
pub fn move_integration_folder(
    git: &Git,
    verb: &str,
    integration_branch: &str,
    old: &Oid,
    new: &Oid,
    actor: &str,
    done: &str,
) -> Result<Option<PathBuf>, Report> {
    if old == new {
        return Ok(None);
    }
    let folder = integration_folder(git, integration_branch).map_err(|reason| Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "folder-not-moved".to_string(),
        lines: vec![done.to_string(), format!("{reason}, so the integration folder was not found and not moved")],
        owner: Some("the repository's worktrees".to_string()),
        next: format!(
            "find the worktree on refs/heads/{integration_branch} (git worktree list), then bring its files and index \
             to {new}, the commit integration now points at; integration itself has already moved"
        ),
        fields: Vec::new(),
    })?;
    let Some(path) = folder else {
        return Ok(None);
    };
    match move_folder(git, &path, old, new, true, actor, None) {
        Ok(()) => Ok(Some(path)),
        Err(error) => Err(folder_move_refusal_report(verb, done, &path, old, new, &why(&error))),
    }
}

/// folder_error_why's lines joined by "; "; the variants it gives no lines for are named in the cli's own words.
fn why(error: &FolderError) -> String {
    match error {
        FolderError::Lock(_) => "its lock could not be taken".to_string(),
        FolderError::Git(_) => "git could not move its files and index".to_string(),
        FolderError::Oplog(_) => "the op log could not record the move".to_string(),
        FolderError::DiskProbe { path, reason } => {
            format!("the free disk at {} could not be read: {reason}", path.display())
        }
        other => folder_error_why(other).join("; "),
    }
}
