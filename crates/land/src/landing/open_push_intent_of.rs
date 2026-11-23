//! open_push_intent_of: the open push intent, if any, in a PushIntentState. Sibling test: open_push_intent_of_tests.

use crate::landing::{OpenPushIntent, PushIntentState};

/// Open { op_id } is Some(OpenPushIntent { op_id }); Absent and Done have no open intent: None.
pub(crate) fn open_push_intent_of(intent: PushIntentState) -> Option<OpenPushIntent> {
    match intent {
        PushIntentState::Open { op_id } => Some(OpenPushIntent { op_id }),
        PushIntentState::Absent | PushIntentState::Done => None,
    }
}
