use super::JsonValue;

fn object() -> JsonValue {
    JsonValue::Object(vec![
        ("outcome".to_string(), JsonValue::Str("empty".to_string())),
        ("records".to_string(), JsonValue::Array(vec![])),
        ("outcome".to_string(), JsonValue::Str("second".to_string())),
    ])
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_member_is_found_by_name_and_the_first_of_two_wins() {
    assert_eq!(object().field("records"), Some(&JsonValue::Array(vec![])));
    assert_eq!(object().field("outcome"), Some(&JsonValue::Str("empty".to_string())));
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_missing_member_or_a_value_that_is_not_an_object_is_none() {
    assert_eq!(object().field("lines"), None);
    assert_eq!(JsonValue::Array(vec![JsonValue::Null]).field("records"), None);
    assert_eq!(JsonValue::Str("records".to_string()).field("records"), None);
}
