//! resolve_landing_state: the landing state as it stands against integration's tip, its pending commit folded in or
//! dropped. resolve_landing_state_tests covers every branch.

use crate::landing::{LandingMember, LandingState};
use gitio::Oid;

/// `integration` is the integration branch's tip as just read. A pending commit at it (commit_files' op ran) becomes
/// the newest member's integration; any other pending commit (its op never ran) is dropped. The answer has no
/// pending.
pub(crate) fn resolve_landing_state(state: LandingState, integration: &Oid) -> LandingState {
    match state.pending {
        Some(pending) if &pending == integration => LandingState {
            newest: LandingMember { integration: pending, ..state.newest },
            pending: None,
            ..state
        },
        _ => LandingState { pending: None, ..state },
    }
}
