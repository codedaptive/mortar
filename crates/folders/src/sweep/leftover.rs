//! A request branch left behind with no folder, and its zone's tip.
//! A bare type; leftover_branches_tests and tests/folders/sweep_pick.rs exercise it.

use gitio::Oid;

/// `branch` is the full name `refs/heads/req/<zone>/<request>` and `tip` its id; `zone_tip` is the id of
/// `refs/heads/zone/<zone>`, against which sweep_refs checks that the branch has joined (S-52 (1)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leftover {
    pub branch: String,
    pub tip: Oid,
    pub zone_tip: Oid,
}
