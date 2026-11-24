//! LandAnswer: what land did. A bare type; land's own landing_zit tests
//! a_squash_on_origin_fast_forwards_integration_to_the_squash, a_squash_behind_a_moved_origin_lands_as_a_merge_commit,
//! a_conflicting_squash_moves_nothing_and_writes_no_state,
//! a_second_squash_of_the_same_zone_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged,
//! old_land_of_another_zone_at_the_tip_is_busy_and_writes_nothing,
//! killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit and
//! killed_after_the_op_recovers_done_and_the_repeat_is_landed_with_no_new_record exercise each variant.

use gitio::Oid;
use oplog::ChangeId;
use std::path::PathBuf;

/// Landed: integration is at `integration` (the squash commit for a fast-forward, else the merge commit), moved by
/// this call or by an earlier one this call repeats. Conflicted: the squash does not merge into integration; these
/// files conflict, integration was not moved, and the next step is back to the zone. Busy: a landing is in flight
/// and this squash is not of it; the change ID is its newest member's, and nothing was moved or written. land never
/// makes a batch of two: another zone's squash while integration is at a landing's tip is Busy; land_batch is the
/// batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LandAnswer {
    Landed { integration: Oid },
    Conflicted { files: Vec<PathBuf> },
    Busy { change_id: ChangeId },
}
