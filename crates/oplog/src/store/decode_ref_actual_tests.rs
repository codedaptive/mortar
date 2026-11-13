//! decode_ref_actual: an id or null actual, and each refusal naming what is wrong.

use gitio::Oid;

use super::{decode_ref_actual, parse_json};

fn decode(text: &str) -> Result<(String, Option<Oid>), String> {
    decode_ref_actual(parse_json(text).unwrap())
}

#[test]
fn entry_with_an_actual_id() {
    assert_eq!(
        decode(r#"{"ref":"refs/heads/a","actual":"cccccccccccccccccccccccccccccccccccccccc"}"#),
        Ok(("refs/heads/a".to_string(), Some(Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap())))
    );
}

#[test]
fn entry_with_a_null_actual_in_any_key_order() {
    assert_eq!(
        decode(r#"{"actual":null,"ref":"refs/heads/b"}"#),
        Ok(("refs/heads/b".to_string(), None))
    );
}

#[test]
fn refusals_name_the_fault() {
    let cases = [
        (r#"{"ref":"refs/heads/a"}"#, "missing key \"actual\""),
        (r#"{"actual":null}"#, "missing key \"ref\""),
        (r#"{"ref":"refs/heads/a","actual":null,"old":null}"#, "the ref entry has unknown key \"old\""),
        (r#"{"ref":"refs/heads/a","ref":"refs/heads/b","actual":null}"#, "the ref entry has key \"ref\" twice"),
        (r#"{"ref":7,"actual":null}"#, "ref is not a string"),
        (r#"{"ref":"refs/heads/a","actual":"HEAD"}"#, "actual \"HEAD\" is not a git object id"),
        (r#""refs/heads/a""#, "the ref entry is not a JSON object"),
    ];
    for (text, want) in cases {
        assert_eq!(decode(text), Err(want.to_string()), "{text}");
    }
}
