//! Moves the integration folder, the worktree on the integration branch, after a verb moved that branch, or when
//! finish_stopped_move finishes a move a stopped run left undone.
//! No sibling test: it runs git and the op log. The RITs crates/cli/tests/cli_rit/run_back_out_tests.rs
//! and crates/cli/tests/cli_rit/run_publish_tests.rs cover it through the binary; folder_move_refusal_report_tests
//! covers the refusal's text. Repo integration tests to write (cross-zone, not written here):
//! - files with local changes in the way in the integration folder when land, back-out or publish (OriginMoved)
//!   moves the integration branch: exit 1 "folder-not-moved" listing the paths, the folder unchanged; move them
//!   out (or put them back to the index copy), then repeat the same command: the repeat's finish moves the folder
//!   and the verb goes on;
//! - the same files in the way at a finish of a stopped move: exit 1 "folder-not-moved" listing the paths,
//!   integration not moved by that run, the folder unchanged; move them out (or put them back to the index copy),
//!   repeat: the move finishes and the verb goes on.
//!
//! Both finish only when the folder was cleared as the next says: a clear that resets the index to HEAD leaves no
//! stall for the repeat to see, and the files are then their owner's, unreported.

use std::path::PathBuf;

use folders::{move_folder, FolderError};
use gitio::{Git, Oid};

use super::{folder_move_refusal_report, integration_folder, MoveBlock};
use crate::folder::folder_error_why;
use crate::report::{ExitCode, Report};

/// `done` is the caller's line saying what already happened (such as "integration is back at <oid>"); `repeat` is
/// the same command run again (the verb, its words and --actor), which every refusal's next ends with. `old` equal
/// to `new` is None with no git run. Else the folder on `integration_branch` (integration_folder): none is None; a
/// worktree list git cannot give is exit 1 "folder-not-moved", owned by the repository's worktrees: `done`, the
/// reason the folder was not found and not moved, and that `verb` stopped; next: check that git can list the
/// worktrees, then repeat. A found folder is moved from `old` to `new` by folders::move_folder with `saved` true,
/// since the integration folder is not an agent's folder; move_folder's plain move still refuses over local
/// changes in the way. A move it refuses is folder_move_refusal_report with the paths in the way, or the cli's
/// words for any other error, and `repeat`. Moved is the folder's path.
#[allow(clippy::too_many_arguments)] // eight inputs, each a distinct fact of the move and its refusal
pub fn move_integration_folder(
    git: &Git,
    verb: &str,
    integration_branch: &str,
    old: &Oid,
    new: &Oid,
    actor: &str,
    done: &str,
    repeat: &str,
) -> Result<Option<PathBuf>, Report> {
    if old == new {
        return Ok(None);
    }
    let folder = integration_folder(git, integration_branch).map_err(|reason| Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "folder-not-moved".to_string(),
        lines: vec![
            done.to_string(),
            format!("{reason}, so the integration folder was not found and not moved"),
            format!("{verb} stopped here"),
        ],
        owner: Some("the repository's worktrees".to_string()),
        next: format!("check that git can list this repository's worktrees, then repeat the same command: {repeat}"),
        fields: Vec::new(),
    })?;
    let Some(path) = folder else {
        return Ok(None);
    };
    match move_folder(git, &path, old, new, true, actor, None) {
        Ok(()) => Ok(Some(path)),
        Err(error) => Err(folder_move_refusal_report(verb, done, &path, old, new, &block(error), repeat)),
    }
}

/// MoveRefused gives its paths; every other error is named in the cli's words: folder_error_why's lines joined by
/// "; ", or the cli's own words for the variants it gives no lines for.
fn block(error: FolderError) -> MoveBlock {
    let why = match error {
        FolderError::MoveRefused { paths, .. } => return MoveBlock::Paths(paths),
        FolderError::Lock(_) => "its lock could not be taken".to_string(),
        FolderError::Git(_) => "git could not move its files and index".to_string(),
        FolderError::Oplog(_) => "the op log could not record the move".to_string(),
        FolderError::DiskProbe { path, reason } => {
            format!("the free disk at {} could not be read: {reason}", path.display())
        }
        other => folder_error_why(&other).join("; "),
    };
    MoveBlock::Other(why)
}
