//! FinishedMove: a stopped integration folder move that finish_stopped_move finished (S-137).
//! A bare type with no behaviour of its own, so no sibling test (S-7).

use std::path::PathBuf;

use gitio::Oid;

/// The integration folder at `folder` moved from `old`, where a stopped run left it, to `tip`, integration's tip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedMove {
    pub folder: PathBuf,
    pub old: Oid,
    pub tip: Oid,
}
