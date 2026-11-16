//! ZoneMove: what after_publish did with one zone. A declaration only (S-7); tests/after_publish_zit (zone_moves.rs)
//! exercises Moved, Current and Conflicted, (records_deleted.rs) the records Moved and Current move, and
//! (older_squash.rs) Current for an older squash published after a newer one.

use gitio::Oid;
use join::OpenRequest;
use std::path::PathBuf;

/// Moved: one op moved `refs/heads/zone/<zone>` from `old` to `new` and, with it, the zone's replay records
/// (S-121): a replay wrote the record of every commit it replayed and carried forward each record naming one of
/// them; the publishing zone deleted every record at or behind its published squash source. `rebase` holds the
/// caller's open requests whose base is not `new`, which rebase at their next join. The zone's folder is the
/// caller's to move (folders move_folder with `old` and `new`, S-47). Current: the zone already holds origin, or the
/// published squash's source is at or behind one already published, so the zone branch did not move; for the
/// publishing zone one records-only op deleted the record keyed by that source and, only when the zone holds that
/// source (itself or through a record), the records at or behind it (S-129), and no op ran when nothing was
/// deleted. Conflicted: replaying the zone onto origin stopped at `commit`, whose `files` conflict; the zone branch
/// and its records stay as they were.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZoneMove {
    Moved { zone: String, old: Oid, new: Oid, rebase: Vec<OpenRequest> },
    Current { zone: String, tip: Oid },
    Conflicted { zone: String, tip: Oid, commit: Oid, files: Vec<PathBuf> },
}
