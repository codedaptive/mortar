//! decode_request_base: hand-written texts read into their bases (no partners; two partners, one zone name escaped);
//! a round trip through encode_request_base over several bases; and each refusal with its reason: text that is not
//! JSON or ends early, a member missing, extra or out of order, a base or tip that is not an id, partners that are
//! not an array, a partner missing its tip or out of order, a zone that is not a string.

use crate::decode_request_base::decode_request_base;
use crate::encode_request_base::encode_request_base;
use crate::partner::Partner;
use crate::request_base::RequestBase;
use gitio::Oid;

const ONE: &str = "1111111111111111111111111111111111111111";
const TWO: &str = "2222222222222222222222222222222222222222";
const THREE: &str = "3333333333333333333333333333333333333333";
const LONG: &str = "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef0123";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn partner(zone: &str, tip: &str) -> Partner {
    Partner { zone: zone.to_string(), tip: oid(tip) }
}

#[test]
fn decode_request_base_reads_the_layout_and_refuses_anything_else() {
    let read = [
        (
            "no partners",
            "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[]}",
            RequestBase { base: oid(ONE), partners: vec![] },
        ),
        (
            "two partners in order, a zone name escaped",
            concat!(
                "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[",
                "{\"zone\":\"we\\\"st\\\\x\",\"tip\":\"2222222222222222222222222222222222222222\"},",
                "{\"zone\":\"east\",\"tip\":\"3333333333333333333333333333333333333333\"}]}"
            ),
            RequestBase { base: oid(ONE), partners: vec![partner("we\"st\\x", TWO), partner("east", THREE)] },
        ),
    ];
    for (case, text, expected) in read {
        assert_eq!(decode_request_base(text), Ok(expected), "{case}");
    }

    let not_an_object = "the base is not an object with exactly the members base then partners";
    let refused = [
        ("empty text", "", "the text ends before the base is complete"),
        (
            "cut short",
            "{\"base\":\"1111111111111111111111111111111111111111\"",
            "the text ends before the base is complete",
        ),
        ("not JSON", "{\"base\" \"1\"}", "the text is not JSON: unexpected '\"' at byte 8 after a key"),
        ("an array", "[]", not_an_object),
        ("partners missing", "{\"base\":\"1111111111111111111111111111111111111111\"}", not_an_object),
        ("base missing", "{\"partners\":[]}", not_an_object),
        (
            "an extra member",
            "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[],\"zone\":\"a\"}",
            not_an_object,
        ),
        ("members reordered", "{\"partners\":[],\"base\":\"1111111111111111111111111111111111111111\"}", not_an_object),
        ("a bad base id", "{\"base\":\"123\",\"partners\":[]}", "base \"123\" is not a git object id"),
        ("a base that is a number", "{\"base\":1,\"partners\":[]}", "base is not a string"),
        (
            "partners not an array",
            "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":{}}",
            "partners is not an array",
        ),
        (
            "a partner missing its tip",
            "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[{\"zone\":\"a\"}]}",
            "partner 0 is not an object with exactly the members zone then tip",
        ),
        (
            "a second partner reordered",
            concat!(
                "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[",
                "{\"zone\":\"a\",\"tip\":\"2222222222222222222222222222222222222222\"},",
                "{\"tip\":\"3333333333333333333333333333333333333333\",\"zone\":\"b\"}]}"
            ),
            "partner 1 is not an object with exactly the members zone then tip",
        ),
        (
            "a bad partner tip",
            "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[{\"zone\":\"a\",\"tip\":\"xyz\"}]}",
            "partner 0's tip \"xyz\" is not a git object id",
        ),
        (
            "a zone that is not a string",
            concat!(
                "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[",
                "{\"zone\":null,\"tip\":\"2222222222222222222222222222222222222222\"}]}"
            ),
            "partner 0's zone is not a string",
        ),
    ];
    for (case, text, reason) in refused {
        assert_eq!(decode_request_base(text), Err(reason.to_string()), "{case}");
    }

    let round_trip = [
        RequestBase { base: oid(TWO), partners: vec![] },
        RequestBase { base: oid(LONG), partners: vec![partner("join", ONE)] },
        RequestBase { base: oid(ONE), partners: vec![partner("a\nb\u{1}\"", TWO), partner("ünï", LONG)] },
        RequestBase { base: oid(THREE), partners: vec![partner("", ONE), partner("x", ONE), partner("x", TWO)] },
    ];
    for base in round_trip {
        assert_eq!(decode_request_base(&encode_request_base(&base)), Ok(base.clone()), "{base:?}");
    }
}
