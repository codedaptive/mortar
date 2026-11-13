//! decode_ref_move: the three keys, old null or an id, each refusal naming what is wrong.

use gitio::Oid;

use super::{decode_ref_move, parse_json, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn decode(text: &str) -> Result<RefMove, String> {
    decode_ref_move(parse_json(text).unwrap())
}

#[test]
fn move_with_old() {
    assert_eq!(
        decode(r#"{"ref":"refs/heads/main","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#),
        Ok(RefMove {
            refname: "refs/heads/main".to_string(),
            old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
            new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        })
    );
}

#[test]
fn create_has_null_old_and_delete_keeps_zero_new() {
    assert_eq!(
        decode(r#"{"new":"0000000000000000000000000000000000000000","old":null,"ref":"refs/tags/v1"}"#),
        Ok(RefMove {
            refname: "refs/tags/v1".to_string(),
            old: None,
            new: oid("0000000000000000000000000000000000000000"),
        })
    );
}

#[test]
fn refusals_name_the_fault() {
    let cases = [
        (r#"{"ref":"r","old":null}"#, "missing key \"new\""),
        (r#"{"ref":"r","new":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#, "missing key \"old\""),
        (r#"{"ref":"r","old":null,"new":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","x":1}"#, "the move has unknown key \"x\""),
        (r#"{"ref":"r","ref":"s","old":null,"new":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#, "the move has key \"ref\" twice"),
        (r#"{"ref":"r","old":null,"new":"xyz"}"#, "new \"xyz\" is not a git object id"),
        (r#"{"ref":1,"old":null,"new":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#, "ref is not a string"),
        (r#"["r",null,"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]"#, "the move is not a JSON object"),
    ];
    for (text, want) in cases {
        assert_eq!(decode(text), Err(want.to_string()), "{text}");
    }
}
