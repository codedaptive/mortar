//! BackOutAnswer: what back_out did. A bare type; land's own landing_zit test
//! back_out_restores_integration_then_has_nothing_to_back_out exercises both variants.

use oplog::ChangeId;

/// BackedOut: integration was moved back to origin and the landing state removed; the landing's newest member had
/// this change ID, which its squash keeps. NothingToBackOut: integration was already at origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackOutAnswer {
    BackedOut { change_id: ChangeId },
    NothingToBackOut,
}
