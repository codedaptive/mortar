//! decode_op_result: each outcome, and the refusals for unknown or ill-formed outcomes.

use gitio::Oid;

use super::{decode_op_result, parse_json, OpResult};

fn decode(text: &str) -> Result<OpResult, String> {
    decode_op_result(parse_json(text).unwrap())
}

#[test]
fn moved() {
    assert_eq!(decode(r#"{"outcome":"moved"}"#), Ok(OpResult::Moved));
}

#[test]
fn not_moved_with_empty_refs() {
    assert_eq!(decode(r#"{"outcome":"not-moved","refs":[]}"#), Ok(OpResult::NotMoved { refs: Vec::new() }));
}

#[test]
fn not_moved_with_two_refs_one_absent() {
    assert_eq!(
        decode(r#"{"outcome":"not-moved","refs":[{"ref":"refs/heads/a","actual":"cccccccccccccccccccccccccccccccccccccccc"},{"ref":"refs/heads/b","actual":null}]}"#),
        Ok(OpResult::NotMoved {
            refs: vec![
                ("refs/heads/a".to_string(), Some(Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap())),
                ("refs/heads/b".to_string(), None),
            ],
        })
    );
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
        (r#"{"outcome":"moved","ref":"r"}"#, "outcome moved takes no ref, actual or refs"),
        (r#"{"outcome":"moved","refs":[]}"#, "outcome moved takes no ref, actual or refs"),
        (r#"{"outcome":"stale","ref":"r","actual":null,"refs":[]}"#, "outcome stale takes no refs"),
        (r#"{"outcome":"not-moved","refs":[],"actual":null}"#, "outcome not-moved takes no ref or actual"),
        (r#"{"outcome":"not-moved","refs":[],"ref":"r"}"#, "outcome not-moved takes no ref or actual"),
        (r#"{"outcome":"not-moved"}"#, "missing key \"refs\""),
        (r#"{"outcome":"not-moved","refs":{}}"#, "refs is not an array"),
        (
            r#"{"outcome":"not-moved","refs":[{"ref":"a","actual":null},{"ref":"b","actual":null,"old":null}]}"#,
            "ref entry 2: the ref entry has unknown key \"old\"",
        ),
        (r#"{"outcome":"not-moved","refs":[{"ref":"a"}]}"#, "ref entry 1: missing key \"actual\""),
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
