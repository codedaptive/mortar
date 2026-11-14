//! JoinAnswer: what a join did. A declaration only; join_answer_display_tests exercises it.

use gitio::Oid;
use std::path::PathBuf;

/// The outcome of a join that was not refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JoinAnswer {
    /// The request tip is already in the zone; no ref moved.
    Joined { zone_ref: String, request_tip: Oid },
    /// The zone ref moved forward from the zone tip to the request tip.
    FastForwarded { zone_ref: String, from: Oid, to: Oid },
    /// The request ref moved from its old tip to its commits replayed onto the zone tip; the zone did not
    /// move. The caller moves the request's folder to `to`; its unit tests run again before the next join.
    Rebased { request_ref: String, from: Oid, to: Oid },
    /// A replayed commit did not apply; no ref moved. `files` are the conflicted paths and `state` is the
    /// mortar-join.json that `mortar resolve` continues from.
    Conflicted { commit: Oid, files: Vec<PathBuf>, state: PathBuf },
}
