//! OpenPushIntent: a publish push intent left open, as the landing status reports it. A bare type with no sibling
//! test (S-7); open_push_intent_display_tests covers its text and publish_zit's
//! status::killed_inside_the_push_the_landing_status_names_the_open_intent reads one from a real op log.

/// The op log record `op_id` of a push intent still open: a publish was killed inside its push, so whether the
/// remote moved is not yet recorded. mr publish reads the remote and closes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenPushIntent {
    pub op_id: String,
}
