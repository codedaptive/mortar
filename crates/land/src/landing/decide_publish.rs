//! decide_publish: which step publish takes, from the remote and local origin tips and the landing state.
//! decide_publish_tests covers every branch.

use crate::landing::{LandingState, PublishStep};
use gitio::Oid;

/// With expected = the state's origin_at_start and new = its integration, in order: no state: Nothing. `local` at
/// new: Cleanup (step 2's origin op ran; the squash ref's removal or the state's did not). `local` at expected, by
/// `remote`: at new, Advance (a publish killed between step 1 and step 2: the push is skipped); at expected, Push;
/// elsewhere, FetchOrigin; absent, RemoteMissing. `local` anywhere else: BackOut onto it (a publish that found the
/// remote moved was killed after fast-forwarding the local origin). `remote` is read only on the `local` at expected
/// branch, so publish passes None on the others.
pub(crate) fn decide_publish(remote: Option<&Oid>, local: &Oid, state: Option<&LandingState>) -> PublishStep {
    let Some(state) = state else {
        return PublishStep::Nothing;
    };
    if local == &state.integration {
        return PublishStep::Cleanup;
    }
    if local != &state.origin_at_start {
        return PublishStep::BackOut { onto: local.clone() };
    }
    match remote {
        None => PublishStep::RemoteMissing,
        Some(remote) if remote == &state.integration => PublishStep::Advance,
        Some(remote) if remote == &state.origin_at_start => PublishStep::Push,
        Some(_) => PublishStep::FetchOrigin,
    }
}
