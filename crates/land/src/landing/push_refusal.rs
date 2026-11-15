//! push_refusal: the refusal among a push's per-ref outcomes. push_refusal_tests covers it.

use gitio::{PushOutcome, PushRefusal};

/// The first Refused outcome's refusal, in the outcomes' order; None when every ref was Pushed or UpToDate.
pub(crate) fn push_refusal(outcomes: Vec<PushOutcome>) -> Option<PushRefusal> {
    outcomes.into_iter().find_map(|outcome| match outcome {
        PushOutcome::Refused(refusal) => Some(refusal),
        PushOutcome::Pushed | PushOutcome::UpToDate => None,
    })
}
