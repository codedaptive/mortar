//! ZoneMove: what after_publish did with one zone. A declaration only (S-7); tests/after_publish_zit (zone_moves.rs)
//! exercises Moved, Current and Conflicted, and (replay_drop.rs)
//! a_zone_whose_replay_dropped_a_commit_owing_a_record_stays_put_and_moves_at_its_own_publish ReplayDropped.

use gitio::Oid;
use join::OpenRequest;
use std::path::PathBuf;

/// Moved: one op moved `refs/heads/zone/<zone>` from `old` to `new` and, with it, the zone's replay records: moved
/// when the replay carried its squash source, deleted when the publishing zone moved past the source one records
/// (S-118); `rebase` holds the caller's open requests whose base is not `new`, which rebase at their next join.
/// The zone's folder is the caller's to move (folders move_folder with `old` and `new`, S-47). Current: the zone
/// already holds origin, so nothing moved. Conflicted: replaying the zone onto origin stopped at `commit`, whose
/// `files` conflict; the zone branch stays at `tip`.
/// ReplayDropped: replaying the zone onto origin dropped a commit that became empty (or gave a range that is not
/// one chain), so the zone commit standing for its squash source `squash_of` cannot be named; the zone branch stays
/// at `tip`, nothing moved and no op ran, until its own squash publishes and moves it (S-120).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZoneMove {
    Moved { zone: String, old: Oid, new: Oid, rebase: Vec<OpenRequest> },
    Current { zone: String, tip: Oid },
    Conflicted { zone: String, tip: Oid, commit: Oid, files: Vec<PathBuf> },
    ReplayDropped { zone: String, tip: Oid, squash_of: Oid },
}
