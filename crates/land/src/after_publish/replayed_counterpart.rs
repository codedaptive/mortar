//! replayed_counterpart: the zone commit an earlier replay made of a squash's source, from commits already read.
//! replayed_counterpart_tests covers it.

use gitio::{CommitInfo, Oid};

/// The first listed commit of `zone_commits` (oldest first) whose message and author (name, email and date) equal
/// `source`'s; None when none does. Tree and parents are ignored: a replay changes both. join's replay keeps each
/// commit's message (trailers and change ID included) and its author with the raw date on both of its paths, so the
/// replayed commit is found by them across any number of earlier replays. The Mortar-Change-Id alone would not do:
/// every commit of one request shares it (S-34). The oldest wins a tie, because replaying from a later one would
/// skip the joins between them.
pub(crate) fn replayed_counterpart(source: &CommitInfo, zone_commits: &[(Oid, CommitInfo)]) -> Option<Oid> {
    zone_commits
        .iter()
        .find(|(_, info)| info.message == source.message && info.author == source.author)
        .map(|(commit, _)| commit.clone())
}
