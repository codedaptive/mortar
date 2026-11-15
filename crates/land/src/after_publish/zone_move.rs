//! ZoneMove: what after_publish did with one zone. A declaration only (S-7); tests/after_publish_zit (zone_moves.rs)
//! exercises each variant.

use gitio::Oid;
use join::OpenRequest;
use std::path::PathBuf;

/// Moved: one op moved `refs/heads/zone/<zone>` from `old` to `new` (and, with it, the zone's replay record when the
/// replay carried its squash source); `rebase` holds the caller's open requests whose
/// base is not `new`, which rebase at their next join. The zone's folder is the caller's to move (folders
/// move_folder with `old` and `new`, S-47). Current: the zone already holds origin, so nothing moved. Conflicted:
/// replaying the zone onto origin stopped at `commit`, whose `files` conflict; the zone branch stays at `tip`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZoneMove {
    Moved { zone: String, old: Oid, new: Oid, rebase: Vec<OpenRequest> },
    Current { zone: String, tip: Oid },
    Conflicted { zone: String, tip: Oid, commit: Oid, files: Vec<PathBuf> },
}
