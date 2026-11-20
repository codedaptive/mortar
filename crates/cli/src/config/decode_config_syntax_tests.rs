//! decode_config's refusal wording for JSON syntax faults, one fault per text: oplog's parse_json tests assert only the error kind.

use super::decode_config_tests::reason;

#[test]
fn syntax_fault_refusals() {
    assert_eq!(reason("{} x"), "unexpected 'x' at byte 3 after the value");
    assert_eq!(reason("{1:2}"), "unexpected '1' at byte 1 where a key belongs");
    assert_eq!(reason(r#"{"a" 1}"#), "unexpected '1' at byte 5 after a key");
    assert_eq!(reason(r#"{"a":1 "b":2}"#), "unexpected '\"' at byte 7 after a member (',' or '}')");
    assert_eq!(reason("[1 2]"), "unexpected '2' at byte 3 after an item (',' or ']')");
    assert_eq!(reason("nil"), "\"nil\" at byte 0 is not null");
}
