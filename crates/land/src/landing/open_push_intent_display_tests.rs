//! Display for OpenPushIntent: the literal text naming the op id and the next command.

use super::OpenPushIntent;

#[test]
fn open_push_intent_names_the_record_and_the_next_command() {
    assert_eq!(
        OpenPushIntent { op_id: "op-7".to_string() }.to_string(),
        "push intent op-7 is still open: a publish was interrupted; next: mr publish"
    );
}
