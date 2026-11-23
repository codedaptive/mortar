//! BaseJoinAnswer: what a join on a request's base did. A declaration only; tests/partner_base exercises each
//! variant through join_on_base.

use crate::request_base::RequestBase;
use crate::JoinAnswer;
use gitio::Oid;
use std::path::PathBuf;

/// The answer of join_on_base.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BaseJoinAnswer {
    /// join's answer, unchanged: the base has no partners, the request is already in the zone, or a replay paused on
    /// a conflict.
    Plain(JoinAnswer),
    /// Zone and partners were where the base had them: one op moved the zone ref from `from` and the request ref
    /// from `request_from`, both to `to`. No stop.
    ZoneJoined { zone_ref: String, request_ref: String, from: Oid, request_from: Oid, to: Oid },
    /// The zone or a partner moved: the request ref moved from `from` to `to`, its own commits replayed onto the new
    /// partner merge `base.base`. The caller rewrites the folder record to `base`, moves the folder from `from` to
    /// `to`, and stops for the unit tests.
    Rebased { request_ref: String, from: Oid, to: Oid, base: RequestBase },
    /// Making the new partner merge conflicted merging partner `zone` on `files`; nothing moved, no state written.
    PartnersConflicted { zone: String, files: Vec<PathBuf> },
}
