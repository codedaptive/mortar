//! paused_join: the join paused in a folder, if any, read from its mortar-join.json (S-84).
//! No unit test: it reads a real file (S-79); the JoinState -> PausedJoin step is a field copy with no decision.
//! The no-state, valid-state, corrupt and unreadable cases are ZITs in crates/join/tests/paused_join.rs (WO-join-6z).
//! Nagatha's integration test is to cover, in a throwaway repository (crosses into gitio; S-84): after a real join
//! that stopped on a conflict, paused_join answers that state's commit and this round's conflicted files.

use crate::join_state_path::join_state_path;
use crate::read_join_state::read_join_state;
use crate::{JoinRunError, PausedJoin};
use std::path::Path;

/// The paused join in the folder whose git dir is `git_dir`, or None when no join is paused there. Read only, for
/// the cli's `mortar status` (S-84): it writes nothing, takes no lock and runs no git. A state file that cannot be
/// read is StateRead; one not in the layout join writes is StateCorrupt.
pub fn paused_join(git_dir: &Path) -> Result<Option<PausedJoin>, JoinRunError> {
    Ok(read_join_state(&join_state_path(git_dir))?.map(|state| PausedJoin { commit: state.commit, files: state.files }))
}
