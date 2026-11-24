//! back_out: moves integration back to where the landing found it, through back_out_batch, answering in one
//! landing's terms; back_out_onto's doc is the contract.
//! No unit test: it maps back_out_batch's answer, which runs git; covered by the landing_zit test
//! backing_out::back_out_restores_integration_then_has_nothing_to_back_out.

use crate::landing::{back_out_batch, BackOutAnswer, BatchBackOutAnswer, LandError};
use crate::squash::LandRefs;
use gitio::Git;

/// back_out_batch(git, refs, actor), its answer as a BackOutAnswer: BackedOut with the newest member's change ID,
/// NothingToBackOut as it is.
pub fn back_out(git: &Git, refs: &LandRefs, actor: &str) -> Result<BackOutAnswer, LandError> {
    Ok(match back_out_batch(git, refs, actor)? {
        BatchBackOutAnswer::BackedOut { change_id, .. } => BackOutAnswer::BackedOut { change_id },
        BatchBackOutAnswer::NothingToBackOut => BackOutAnswer::NothingToBackOut,
    })
}
