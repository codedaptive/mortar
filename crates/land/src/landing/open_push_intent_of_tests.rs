//! open_push_intent_of: Open carries its op id out; Absent and Done carry no open intent.

use super::{open_push_intent_of, OpenPushIntent, PushIntentState};

#[test]
fn open_push_intent_of_each_push_intent_state() {
    let rows = [
        ("open", PushIntentState::Open { op_id: "op-7".to_string() }, Some(OpenPushIntent { op_id: "op-7".to_string() })),
        ("absent", PushIntentState::Absent, None),
        ("done", PushIntentState::Done, None),
    ];
    let failures: Vec<_> = rows
        .into_iter()
        .filter_map(|(name, intent, expected)| {
            let actual = open_push_intent_of(intent);
            (actual != expected).then(|| format!("{name}: expected {expected:?}, got {actual:?}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
