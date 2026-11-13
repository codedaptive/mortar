//! decode_op_result: each outcome, and the refusals for unknown or ill-formed outcomes.

use gitio::Oid;

use super::{decode_op_result, parse_json, OpResult};

fn decode(text: &str) -> Result<OpResult, String> {
    decode_op_result(parse_json(text).unwrap())
}

#[test]
fn moved_and_not_moved() {
    assert_eq!(decode(r#"{"outcome":"moved"}"#), Ok(OpResult::Moved));
    assert_eq!(decode(r#"{"outcome":"not-moved"}"#), Ok(OpResult::NotMoved));
}

#[test]
fn stale_with_actual_id_and_with_null() {
    assert_eq!(
        decode(r#"{"outcome":"stale","ref":"refs/heads/x","actual":"cccccccccccccccccccccccccccccccccccccccc"}"#),
        Ok(OpResult::Stale {
            refname: "refs/heads/x".to_string(),
            actual: Some(Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap()),
        })
    );
    assert_eq!(
        decode(r#"{"actual":null,"ref":"refs/heads/x","outcome":"stale"}"#),
        Ok(OpResult::Stale {
            refname: "refs/heads/x".to_string(),
            actual: None,
        })
    );
}

#[test]
fn refusals_name_the_fault() {
    let cases = [
        (r#"{"outcome":"Moved"}"#, "unknown outcome \"Moved\" (expected moved, stale or not-moved)"),
        (r#"{"outcome":"moved","ref":"r"}"#, "outcome moved takes no ref or actual"),
        (r#"{"outcome":"not-moved","actual":null}"#, "outcome not-moved takes no ref or actual"),
        (r#"{"outcome":"stale","ref":"r"}"#, "missing key \"actual\""),
        (r#"{"outcome":"stale","actual":null}"#, "missing key \"ref\""),
        (r#"{}"#, "missing key \"outcome\""),
        (r#"{"outcome":"moved","why":"x"}"#, "the result has unknown key \"why\""),
        (r#""moved""#, "the result is not a JSON object"),
    ];
    for (text, want) in cases {
        assert_eq!(decode(text), Err(want.to_string()), "{text}");
    }
}
