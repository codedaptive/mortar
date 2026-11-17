//! RestoredJoin: what restore_join_state answers. A declaration only; restore_join_state's header lists the
//! integration cases that exercise it.

use crate::PausedJoin;
use std::path::PathBuf;

/// `state` is the path of mortar-join.json; `paused` is the join it now describes, as paused_join answers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoredJoin {
    /// The recorded state was written as mortar-join.json.
    Restored { state: PathBuf, paused: PausedJoin },
    /// mortar-join.json already held the recorded state (a repeat); nothing was written.
    AlreadyRestored { state: PathBuf, paused: PausedJoin },
}
