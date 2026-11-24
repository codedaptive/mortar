//! BatchBackOutAnswer: what back_out_batch did. A bare type with no sibling test (S-7); the landing_zit tests
//! batch::back_out_restores_origin_for_every_member and kills::killed_second_member_recovers_then_back_out_discards_both
//! exercise BackedOut through back_out_batch, and backing_out::back_out_restores_integration_then_has_nothing_to_back_out
//! both variants through back_out, which maps it onto BackOutAnswer.

use oplog::ChangeId;

/// BackedOut: integration was moved back to origin and the landing state removed; `change_id` is the newest
/// member's and `change_ids` every member's in landing order, which their squashes keep. NothingToBackOut:
/// integration was already at origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchBackOutAnswer {
    BackedOut { change_id: ChangeId, change_ids: Vec<ChangeId> },
    NothingToBackOut,
}
