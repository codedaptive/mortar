//! Zone integration tests of after_publish (SPEC-land 05-after-publish.md §Tests): another zone replayed onto the
//! new origin, the publishing zone with and without later joins, a conflicting zone left in place and listed, a
//! repeat that moves nothing (zone_moves), and the publishing zone landed as a merge (merge_landing); each case in
//! its own throwaway local repository with a throwaway bare remote, the published state reached with land's own
//! squash, land and publish.

#[path = "after_publish_zit/fixture.rs"]
mod fixture;
#[path = "after_publish_zit/merge_landing.rs"]
mod merge_landing;
#[path = "after_publish_zit/zone_moves.rs"]
mod zone_moves;
