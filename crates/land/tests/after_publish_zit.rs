//! Zone integration tests of after_publish (SPEC-land 05-after-publish.md §Tests, S-121): another zone replayed onto
//! the new origin with the record of its replayed commit written, the publishing zone with and without later joins,
//! a conflicting zone left in place and listed, a repeat that moves nothing (zone_moves); the publishing zone landed
//! as a merge (merge_landing); the publishing zone whose squash source an earlier after_publish replayed, with and
//! without a later join (replayed_source); that source told apart by the replay rather than by message, author and
//! date: two saves sharing them, a re-save sharing them refused, the source object pruned (replay_provenance); the
//! replay record deleted by the publish that consumes it, with the zone put on origin or replayed onto it, and left
//! alone by a repeat (record_consumed); a zone whose replay drops a commit moved and the dropped commit recorded as
//! the commit it collapsed into, with git's replay probe as installed and with it forced unavailable (replay_drop); a zone
//! ref restored behind its record, rewritten by the next replay (restored_zone_ref); an older squash published
//! after a newer one replaced the squash ref, with every replayed commit recorded (squash_ref_replaced); a held
//! record read without reading the source (source_not_read); the records at or behind the published source deleted
//! in the zone's op, and by a records-only op when the zone is Current (records_deleted); an older squash behind the
//! published one left Current, and one not behind it refused (older_squash). Each case in its own throwaway local
//! repository with a throwaway bare remote, the published state reached with land's own squash, land and publish;
//! fixture holds the shared arrangements and shims the git shims.

#[path = "after_publish_zit/fixture.rs"]
mod fixture;
#[path = "after_publish_zit/merge_landing.rs"]
mod merge_landing;
#[path = "after_publish_zit/older_squash.rs"]
mod older_squash;
#[path = "after_publish_zit/record_consumed.rs"]
mod record_consumed;
#[path = "after_publish_zit/records_deleted.rs"]
mod records_deleted;
#[path = "after_publish_zit/replay_drop.rs"]
mod replay_drop;
#[path = "after_publish_zit/replay_provenance.rs"]
mod replay_provenance;
#[path = "after_publish_zit/replayed_source.rs"]
mod replayed_source;
#[path = "after_publish_zit/restored_zone_ref.rs"]
mod restored_zone_ref;
#[path = "after_publish_zit/shims.rs"]
mod shims;
#[path = "after_publish_zit/source_not_read.rs"]
mod source_not_read;
#[path = "after_publish_zit/squash_ref_replaced.rs"]
mod squash_ref_replaced;
#[path = "after_publish_zit/zone_moves.rs"]
mod zone_moves;
