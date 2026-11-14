//! PausedJoin: what `mortar status` is told about a join paused on a conflict. A declaration only; paused_join's
//! header lists the integration cases that exercise it. Not named paused_join.rs: the function paused_join holds
//! that name.

use gitio::Oid;
use std::path::PathBuf;

/// The commit a paused join stopped at, and its conflicted files, relative to the folder (S-21, S-84).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PausedJoin {
    pub commit: Oid,
    pub files: Vec<PathBuf>,
}
