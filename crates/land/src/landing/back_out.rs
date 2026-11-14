//! back_out: moves integration back to where the landing found it (SPEC-land 03-land.md, S-56, S-59), through
//! back_out_onto with no origin given; back_out_onto's doc is the contract.
//! No unit test (S-5): it is one call; covered by the landing_zit test
//! back_out_restores_integration_then_has_nothing_to_back_out.

use crate::landing::{back_out_onto, BackOutAnswer, LandError};
use crate::squash::LandRefs;
use gitio::Git;

/// back_out_onto(git, refs, actor, None): integration goes back to the landing state's origin_at_start, read
/// against the local origin tip.
pub fn back_out(git: &Git, refs: &LandRefs, actor: &str) -> Result<BackOutAnswer, LandError> {
    back_out_onto(git, refs, actor, None)
}
