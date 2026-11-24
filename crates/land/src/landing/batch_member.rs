//! BatchMember: one squash of the landing batch as read_landing_batch names it. A bare type with no sibling test
//! (S-7); batch_members_tests builds and compares it, and landing_zit's
//! batch::read_landing_batch_names_the_members_in_landing_order reads it from a real repository.

use oplog::ChangeId;

/// The change ID and zone of a squash landed in the batch: which member a landing is, and the zones a publish is
/// for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchMember {
    pub change_id: ChangeId,
    pub zone: String,
}
