//! LandingStatus: the landing in flight as the landing status reports it. A bare type with no sibling test (S-7);
//! publish_zit's status::killed_inside_the_push_the_landing_status_names_the_open_intent and
//! status::with_no_publish_in_progress_the_landing_status_carries_no_intent read it from real repositories.

use crate::landing::OpenPushIntent;
use gitio::Oid;
use oplog::ChangeId;

/// The squash being landed (its change ID, zone and commit), the origin tip when the landing started, and the
/// commit integration moves to, resolved: a pending commit_files commit folded in when integration reached it, else
/// dropped. `open_push` is Some while this landing's publish push intent is open (a publish killed inside its push;
/// next: mr publish), else None.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandingStatus {
    pub change_id: ChangeId,
    pub zone: String,
    pub squash: Oid,
    pub origin_at_start: Oid,
    pub integration: Oid,
    pub open_push: Option<OpenPushIntent>,
}
