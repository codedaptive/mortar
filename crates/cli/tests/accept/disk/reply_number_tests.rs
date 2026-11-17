use super::reply_number;
use crate::json::JsonValue;
use crate::stage::Reply;

fn reply(report: JsonValue) -> Reply {
    Reply { exit: Some(0), outcome: "listed".to_string(), next: String::new(), lines: vec![], report, saw: "exit 0, listed".to_string() }
}

#[test]
fn a_top_level_u64_number_is_returned() {
    let report = JsonValue::Object(vec![("store_bytes".to_string(), JsonValue::Number("18446744073709551615".to_string()))]);
    assert_eq!(reply_number(&reply(report), "store_bytes"), Ok(18_446_744_073_709_551_615));
}

#[test]
fn missing_null_and_other_json_kinds_report_no_number() {
    for report in [
        JsonValue::Object(vec![]),
        JsonValue::Object(vec![("store_bytes".to_string(), JsonValue::Null)]),
        JsonValue::Object(vec![("store_bytes".to_string(), JsonValue::Str("42".to_string()))]),
        JsonValue::Object(vec![("store_bytes".to_string(), JsonValue::Bool(true))]),
        JsonValue::Object(vec![("store_bytes".to_string(), JsonValue::Array(vec![]))]),
        JsonValue::Object(vec![("nested".to_string(), JsonValue::Object(vec![("store_bytes".to_string(), JsonValue::Number("42".to_string()))]))]),
        JsonValue::Null,
    ] {
        assert_eq!(reply_number(&reply(report), "store_bytes"), Err("the reply has no number store_bytes: exit 0, listed".to_string()));
    }
}

#[test]
fn negative_fractional_and_overflow_numbers_are_not_whole_u64_numbers() {
    for (text, error) in [
        ("-1", "store_bytes is not a whole number: -1"),
        ("1.5", "store_bytes is not a whole number: 1.5"),
        ("18446744073709551616", "store_bytes is not a whole number: 18446744073709551616"),
    ] {
        let report = JsonValue::Object(vec![("store_bytes".to_string(), JsonValue::Number(text.to_string()))]);
        assert_eq!(reply_number(&reply(report), "store_bytes"), Err(error.to_string()));
    }
}
