//! paused_join: the join paused in a folder, if any, read from its mortar-join.json.
//! No unit test: it reads a real file; the JoinState -> PausedJoin step is a field copy with no decision.
//! The no-state, valid-state, corrupt and unreadable cases are ZITs in crates/join/tests/paused_join.rs.
//! A real join stopped on a conflict is covered by the ZIT a_join_paused_on_a_conflict_is_answered_by_paused_join in
//! crates/join/tests/real_join.rs.

use crate::join_state_path::join_state_path;
use crate::read_join_state::read_join_state;
use crate::{JoinRunError, PausedJoin};
use std::path::Path;

/// The paused join in the folder whose git dir is `git_dir`, or None when no join is paused there. Read only, for
/// the cli's `mr status`: it writes nothing, takes no lock and runs no git. A state file that cannot be
/// read is StateRead; one not in the layout join writes is StateCorrupt.
pub fn paused_join(git_dir: &Path) -> Result<Option<PausedJoin>, JoinRunError> {
    Ok(read_join_state(&join_state_path(git_dir))?.map(|state| PausedJoin { commit: state.commit, files: state.files }))
}
