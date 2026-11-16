//! Squash: one zone squash commit. A declaration only (S-7); tests/squash_zit.rs exercises it.
//! Not named squash.rs: that file holds the function of the same name.

use gitio::Oid;
use oplog::ChangeId;

/// The commit at `refs/mortar/squash/<zone>`, its change ID, the zone tip it squashes and the requests it
/// squashes: the Mortar-Request values the squash message's request lines name, in message order (a squash
/// carries no Mortar-Request trailer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Squash {
    pub commit: Oid,
    pub change_id: ChangeId,
    pub source: Oid,
    pub requests: Vec<String>,
}
