//! PushIntentState: where this publish's push intent stands in the op log, as read_push_intent reads it. A bare type
//! with no sibling test (S-7); decide_publish_tests takes each variant, and the publish_zit repeats tests
//! open_push_intent_refuses_push_off_then_push_on_finishes and
//! push_off_killed_after_origin_move_finishes_push_on_without_push read it from a real log.

/// Absent: no push intent for this move, or only failed ones. Open: an intent left open (a push-on run killed
/// between its steps), the record `op_id`. Done: a push intent for this move closed done, so the remote was at or
/// past the landed commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PushIntentState {
    Absent,
    Open { op_id: String },
    Done,
}
