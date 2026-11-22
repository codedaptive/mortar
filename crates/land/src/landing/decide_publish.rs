//! decide_publish: which step publish takes, from push on or off, the remote and local origin tips and the landing
//! state. decide_publish_tests covers every branch.

use crate::landing::{LandingState, PublishStep};
use gitio::Oid;

/// With expected = the state's origin_at_start and new = its integration, in order: no state: Nothing. `local` at
/// new: Cleanup (step 2's origin op ran; the squash ref's removal or the state's did not). `local` anywhere but
/// expected: BackOut onto it (push on: a publish that found the remote moved was killed after fast-forwarding the
/// local origin; push off: the local origin moved without us). `local` at expected with push off: Advance, step 2
/// without the remote. `local` at expected with push on, by `remote`: at new, Advance (a publish killed between
/// step 1 and step 2: the push is skipped); at expected, Push; elsewhere, FetchOrigin; absent, RemoteMissing.
/// `remote` is read only on the push-on, `local` at expected branch, so publish_with_push passes None on the
/// others; with push off it never changes the answer.
pub(crate) fn decide_publish(
    push: bool,
    remote: Option<&Oid>,
    local: &Oid,
    state: Option<&LandingState>,
) -> PublishStep {
    let Some(state) = state else {
        return PublishStep::Nothing;
    };
    if local == &state.integration {
        return PublishStep::Cleanup;
    }
    if local != &state.origin_at_start {
        return PublishStep::BackOut { onto: local.clone() };
    }
    if !push {
        return PublishStep::Advance;
    }
    match remote {
        None => PublishStep::RemoteMissing,
        Some(remote) if remote == &state.integration => PublishStep::Advance,
        Some(remote) if remote == &state.origin_at_start => PublishStep::Push,
        Some(_) => PublishStep::FetchOrigin,
    }
}
