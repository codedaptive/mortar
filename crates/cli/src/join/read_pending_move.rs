//! Reads the op log with each record's values and the request branch, and asks find_pending_move whether a folder
//! move is pending, a join rebase's, a joined partner request's or an undo's of a join rebase that is not a resolve
//! step (an undo of a resolve step is none), and from which recorded folder_tree; and asks pending_base, from the same
//! read, the base the folder record must hold once that move is finished.
//! finish_pending_move (join, resolve, save, undo) calls it, after recover has settled open intents, and
//! follow_answer calls it after join's plain rebase op for the base that op recorded.
//! No unit test: it reads the op log file and runs git; find_pending_move_tests and pending_base_tests cover the
//! decisions.
//! The integration tests crates/cli/tests/save_join_runs/{recovery,processes}.rs, resolve_runs/resolve_pending.rs and
//! across_kits/undo_save_join_resolve/{rebases,resolve_undo}.rs cover, in a throwaway repository with a request
//! folder: a join-rebase op
//! done and the folder still at the old tip gives the move from the op's old tip to its new; after the folder
//! move it still gives that move (move_folder logs no op then); after a later save of the request branch it gives
//! None; a request branch moved elsewhere by hand gives None; a join-rebase op carrying folder_tree gives the move
//! with that tree; one carrying none gives the move with folder_tree None; no op log file gives None; an op log that
//! cannot be read is exit 5 naming its path; an undo of a plain join-rebase op, the folder still at the rebased tip,
//! gives the move back to the tip before the rebase; after mr undo of a resolve's final join step it
//! gives None, and the next save, join or resolve moves no folder files. It moves no ref and writes no file.

use std::path::Path;

use folders::RecordedBase;
use gitio::Git;
use oplog::read_valued_log;

use super::{find_pending_move, pending_base, PendingMove};

/// A pending move with pending_base's answer for it.
type WithBase = (PendingMove, Result<Option<RecordedBase>, String>);

/// The pending move of `request_ref`, read from the op log under `common` and the ref's value through `git`, with
/// pending_base's answer for it from the same records. Either read failing is its error's text.
pub fn read_pending_move(git: &Git, common: &Path, request_ref: &str) -> Result<Option<WithBase>, String> {
    let log = read_valued_log(common).map_err(|error| error.to_string())?;
    let branch_at = git.read_ref(request_ref).map_err(|error| error.to_string())?;
    Ok(find_pending_move(&log.records, request_ref, branch_at.as_ref()).map(|pending| {
        let base = pending_base(&log.records, &pending);
        (pending, base)
    }))
}
