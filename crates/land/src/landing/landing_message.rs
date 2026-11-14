//! landing_message: the message of a landing's merge commit (S-58).
//! No sibling unit test (S-104): every test of it runs oplog's ChangeId Display (S-101). Covered by tests/landing_zit
//! (landing.rs, a_squash_behind_a_moved_origin_lands_as_a_merge_commit: the merge commit's message); its unit case is
//! one of Nagatha's RITs (owed).

use oplog::ChangeId;

/// `land <change id> (<zone>)` and a newline.
pub(crate) fn landing_message(change_id: &ChangeId, zone: &str) -> String {
    format!("land {change_id} ({zone})\n")
}
