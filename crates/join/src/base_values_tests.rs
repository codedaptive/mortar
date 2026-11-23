//! base_values: the two pairs, base_before then base_after, each holding its base's text written out by hand.

use crate::base_values::base_values;
use crate::partner::Partner;
use crate::request_base::RequestBase;
use gitio::Oid;

#[test]
fn base_values_is_base_before_then_base_after() {
    let before =
        RequestBase { base: Oid::parse("1111111111111111111111111111111111111111").unwrap(), partners: vec![] };
    let after = RequestBase {
        base: Oid::parse("2222222222222222222222222222222222222222").unwrap(),
        partners: vec![Partner {
            zone: "east".to_string(),
            tip: Oid::parse("3333333333333333333333333333333333333333").unwrap(),
        }],
    };
    assert_eq!(
        base_values(&before, &after),
        vec![
            (
                "base_before".to_string(),
                "{\"base\":\"1111111111111111111111111111111111111111\",\"partners\":[]}".to_string()
            ),
            (
                "base_after".to_string(),
                concat!(
                    "{\"base\":\"2222222222222222222222222222222222222222\",\"partners\":[",
                    "{\"zone\":\"east\",\"tip\":\"3333333333333333333333333333333333333333\"}]}"
                )
                .to_string()
            ),
        ]
    );
}
