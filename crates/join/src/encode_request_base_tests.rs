//! encode_request_base: a base with no partners and a base with two partners, one of whose zone names needs JSON
//! escapes, each against its text written out by hand.

use crate::encode_request_base::encode_request_base;
use crate::partner::Partner;
use crate::request_base::RequestBase;
use gitio::Oid;

const ONE: &str = "1111111111111111111111111111111111111111";
const TWO: &str = "2222222222222222222222222222222222222222";
const THREE: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn encode_request_base_writes_base_then_partners_with_no_whitespace() {
    let rows = [
        (
            "no partners",
            RequestBase { base: oid(ONE), partners: vec![] },
            "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[]}",
        ),
        (
            "two partners in order, a zone name escaped",
            RequestBase {
                base: oid(ONE),
                partners: vec![
                    Partner { zone: "we\"st\\x".to_string(), tip: oid(TWO) },
                    Partner { zone: "east".to_string(), tip: oid(THREE) },
                ],
            },
            concat!(
                "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[",
                "{\"zone\":\"we\\\"st\\\\x\",\"tip\":\"2222222222222222222222222222222222222222\"},",
                "{\"zone\":\"east\",\"tip\":\"3333333333333333333333333333333333333333\"}]}"
            ),
        ),
    ];
    for (case, base, expected) in rows {
        assert_eq!(encode_request_base(&base), expected, "{case}");
    }
}
