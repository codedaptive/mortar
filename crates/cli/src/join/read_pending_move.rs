//! Reads the op log with each record's values and the request branch, and asks find_pending_move whether a folder
//! move is pending, a join rebase's or an undo's of a plain join rebase (an undo of a resolve step is
//! none), and from which recorded folder_tree.
//! finish_pending_move (join, resolve, save, undo) calls it, after recover has settled open intents.
//! No unit test: it reads the op log file and runs git; find_pending_move_tests covers the decision.
//! The repo integration tests are to cover, in a throwaway repository with a request folder: a join-rebase op
//! done and the folder still at the old tip gives the move from the op's old tip to its new; after the folder
//! move it still gives that move (move_folder logs no op then); after a later save of the request branch it gives
//! None; a request branch moved elsewhere by hand gives None; a join-rebase op carrying folder_tree gives the move
//! with that tree; one carrying none gives the move with folder_tree None; no op log file gives None; an op log that
//! cannot be read is exit 5 naming its path; an undo of a plain join-rebase op, the folder still at the rebased tip,
//! gives the move back to the tip before the rebase; after mr undo of a resolve's final join step it
//! gives None, and the next save, join or resolve moves no folder files. It moves no ref and writes no file.

use std::path::Path;

use gitio::Git;
use oplog::read_valued_log;

use super::{find_pending_move, PendingMove};
use crate::report::{failure_report, Report};

/// The pending move of `request_ref`, read from the op log under `common` and the ref's value through `git`.
/// Either read failing is failure_report for `verb`.
pub fn read_pending_move(
    git: &Git,
    common: &Path,
    request_ref: &str,
    verb: &str,
) -> Result<Option<PendingMove>, Report> {
    let log = read_valued_log(common).map_err(|error| failure_report(verb, &error.to_string()))?;
    let branch_at = git.read_ref(request_ref).map_err(|error| failure_report(verb, &error.to_string()))?;
    Ok(find_pending_move(&log.records, request_ref, branch_at.as_ref()))
}
