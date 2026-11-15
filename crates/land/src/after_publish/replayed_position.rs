//! replayed_position: the commit a replay made of the zone's commit for a squash source, by its position in the
//! replayed range. replayed_position_tests covers it.

use gitio::Oid;

/// The commit looked up is `recorded` when the source has a replay record, else `source`: a chain of replays
/// follows its record. `old` is the replayed range (oldest first), `old_line` its first-parent line; `new` and
/// `new_line` the same for the replay's result. Some(new[i]) when `old[i]` is that commit, but only when each range
/// is one first-parent chain (`old == old_line`, `new == new_line`) and both have the same length; else None. Under
/// those checks position is sound: join's commit-by-commit path writes one commit per range commit, in order, each
/// on the previous (join replay_commits.rs), and never adds one; git's replay (git 2.56 on this machine) can drop a
/// commit that becomes empty, as the old matching's re-replay of X2' in tests/after_publish_zit replay_provenance.rs
/// case (a) did, giving origin itself back. The length check covers both paths: a drop or any other shape gives no
/// answer, never a wrong one.
pub(crate) fn replayed_position(
    source: &Oid,
    recorded: Option<&Oid>,
    old: &[Oid],
    old_line: &[Oid],
    new: &[Oid],
    new_line: &[Oid],
) -> Option<Oid> {
    if old != old_line || new != new_line || old.len() != new.len() {
        return None;
    }
    let current = recorded.unwrap_or(source);
    old.iter().position(|commit| commit == current).map(|i| new[i].clone())
}
