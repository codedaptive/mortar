//! Zone integration tests of after_publish (SPEC-land 05-after-publish.md §Tests): another zone replayed onto the
//! new origin, the publishing zone with and without later joins, a conflicting zone left in place and listed, a
//! repeat that moves nothing (zone_moves), the publishing zone landed as a merge (merge_landing), and the publishing
//! zone whose squash source an earlier after_publish replayed, with and without a later join (replayed_source), and
//! that source told apart by the replay rather than by message, author and date: two saves sharing them, a re-save
//! sharing them refused, the source object pruned (replay_provenance); each case in its own throwaway local
//! repository with a throwaway bare remote, the published state reached with land's own squash, land and publish.

#[path = "after_publish_zit/fixture.rs"]
mod fixture;
#[path = "after_publish_zit/merge_landing.rs"]
mod merge_landing;
#[path = "after_publish_zit/replay_provenance.rs"]
mod replay_provenance;
#[path = "after_publish_zit/replayed_source.rs"]
mod replayed_source;
#[path = "after_publish_zit/zone_moves.rs"]
mod zone_moves;
