//! JoinRead: what join reads before deciding. A declaration only; decide_join_tests exercises it.

use gitio::Oid;

/// The zone ref's tip, the request ref's tip, and their merge base (None when the histories are unrelated).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct JoinRead {
    pub(crate) zone_tip: Oid,
    pub(crate) request_tip: Oid,
    pub(crate) fork: Option<Oid>,
}
