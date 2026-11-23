//! state_base_after: the base a paused join's request stands on once resolved. state_base_after_tests covers a
//! state with a base and one without.

use crate::join_state::JoinState;
use crate::request_base::RequestBase;

/// The state's base when it kept one (join_on_base's pauses); otherwise the zone tip it replayed onto, `onto`, with
/// no partners, the base a request joined by plain replay stands on.
pub(crate) fn state_base_after(state: &JoinState) -> RequestBase {
    state.base.clone().unwrap_or_else(|| RequestBase { base: state.onto.clone(), partners: vec![] })
}
