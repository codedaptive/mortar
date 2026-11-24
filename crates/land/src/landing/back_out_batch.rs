//! back_out_batch: moves integration back to where the landing batch found it, through back_out_onto with no
//! origin given; back_out_onto's doc is the contract.
//! No unit test: it is one call that runs git; covered by the landing_zit tests
//! batch::back_out_restores_origin_for_every_member and kills::killed_second_member_recovers_then_back_out_discards_both,
//! and through back_out by backing_out::back_out_restores_integration_then_has_nothing_to_back_out.

use crate::landing::{back_out_onto, BatchBackOutAnswer, LandError};
use crate::squash::LandRefs;
use gitio::Git;

/// back_out_onto(git, refs, actor, None): integration goes back to the landing state's origin_at_start, read
/// against the local origin tip; BackedOut names the newest member's change ID and every member's in landing order.
pub fn back_out_batch(git: &Git, refs: &LandRefs, actor: &str) -> Result<BatchBackOutAnswer, LandError> {
    back_out_onto(git, refs, actor, None)
}
