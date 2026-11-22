//! WithdrawAnswer: what withdraw did. A bare type, no sibling test; the withdraw_zit tests
//! a_withdraw_deletes_the_squash_ref_and_answers_its_commit_and_change_id (Withdrawn) and
//! a_repeated_withdraw_is_absent_and_writes_nothing (Absent) exercise it.

use gitio::Oid;
use oplog::ChangeId;

/// A withdraw's answer when nothing refused it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WithdrawAnswer {
    /// The squash ref was deleted: the commit it held and that commit's Mortar-Change-Id (None: it carried none).
    Withdrawn { commit: Oid, change_id: Option<ChangeId> },
    /// There was no squash ref, so nothing was written; a repeat of a withdraw is already done.
    Absent,
}
