//! request_base_of: the base and each partner carried over field for field, partners in order.

use ::join::{Partner, RequestBase};
use folders::{RecordedBase, RequestPartner};
use gitio::Oid;

use super::request_base_of;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

#[test]
fn request_base_of_cases() {
    let cases: Vec<(&str, RecordedBase, RequestBase)> = vec![
        (
            "no partners",
            RecordedBase { base: oid('d'), partners: vec![] },
            RequestBase { base: oid('d'), partners: vec![] },
        ),
        (
            "two partners in order",
            RecordedBase {
                base: oid('d'),
                partners: vec![
                    RequestPartner { zone: "api".to_string(), tip: oid('e') },
                    RequestPartner { zone: "web".to_string(), tip: oid('f') },
                ],
            },
            RequestBase {
                base: oid('d'),
                partners: vec![
                    Partner { zone: "api".to_string(), tip: oid('e') },
                    Partner { zone: "web".to_string(), tip: oid('f') },
                ],
            },
        ),
    ];
    for (name, base, expected) in cases {
        assert_eq!(request_base_of(&base), expected, "case {name}");
    }
}
