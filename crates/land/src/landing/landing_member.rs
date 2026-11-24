//! LandingMember: one squash of the landing batch, as landing.json's `landings` array holds it. A bare type: no
//! sibling test; landing_state_members_tests, encode_landing_state_tests and decode_landing_state_tests build and
//! compare it, and tests/landing_zit exercises it through land and back_out.

use gitio::Oid;
use oplog::ChangeId;

/// The squash landed (its change ID, zone and commit) and the commit its landing moved integration to: the squash
/// itself for a fast-forward, else the merge commit, or a commit commit_files wrote on top of either.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LandingMember {
    pub change_id: ChangeId,
    pub zone: String,
    pub squash: Oid,
    pub integration: Oid,
}
