//! LandAnswer: what land did. A bare type; land's own landing_zit tests
//! a_squash_on_origin_fast_forwards_integration_to_the_squash, a_squash_behind_a_moved_origin_lands_as_a_merge_commit,
//! a_conflicting_squash_moves_nothing_and_writes_no_state,
//! a_second_landing_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged,
//! killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit and
//! killed_after_the_op_recovers_done_and_the_repeat_is_landed_with_no_new_record exercise each variant.

use gitio::Oid;
use oplog::ChangeId;
use std::path::PathBuf;

/// Landed: integration is at `integration` (the squash commit for a fast-forward, else the merge commit), moved by
/// this call or by an earlier one this call repeats. Conflicted: the squash does not merge into integration; these
/// files conflict, integration was not moved, and the next step is back to the zone. Busy: another landing, this
/// change ID's, is in flight; integration was not moved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LandAnswer {
    Landed { integration: Oid },
    Conflicted { files: Vec<PathBuf> },
    Busy { change_id: ChangeId },
}
