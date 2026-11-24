//! BatchLandAnswer: what land_batch did. A bare type with no sibling test (S-7); the landing_zit tests exercise
//! Landed, Conflicted and Busy through land, which maps it onto LandAnswer, and
//! batch::duplicate_zone_reports_its_member_and_changes_nothing exercises ZoneInBatch through land_batch.

use gitio::Oid;
use oplog::ChangeId;
use std::path::PathBuf;

/// Landed: integration is at `integration` (the squash commit for a fast-forward, else the merge commit), moved by
/// this call or by an earlier one this call repeats. Conflicted: the squash does not merge into integration; these
/// files conflict, integration was not moved, and the next step is back to the zone. Busy: another batch is in
/// flight, its newest member having this change ID; integration was not moved. ZoneInBatch: a member of the batch
/// already holds this squash's zone, and `change_id` is that member's; nothing was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchLandAnswer {
    Landed { integration: Oid },
    Conflicted { files: Vec<PathBuf> },
    Busy { change_id: ChangeId },
    ZoneInBatch { zone: String, change_id: ChangeId },
}
