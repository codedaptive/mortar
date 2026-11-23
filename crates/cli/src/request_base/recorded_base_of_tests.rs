//! recorded_base_of: the base and each partner carried over field for field, partners in order.

use ::join::{Partner, RequestBase};
use folders::{RecordedBase, RequestPartner};
use gitio::Oid;

use super::recorded_base_of;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

#[test]
fn recorded_base_of_cases() {
    let cases: Vec<(&str, RequestBase, RecordedBase)> = vec![
        (
            "no partners",
            RequestBase { base: oid('a'), partners: vec![] },
            RecordedBase { base: oid('a'), partners: vec![] },
        ),
        (
            "two partners in order",
            RequestBase {
                base: oid('a'),
                partners: vec![
                    Partner { zone: "web".to_string(), tip: oid('b') },
                    Partner { zone: "api".to_string(), tip: oid('c') },
                ],
            },
            RecordedBase {
                base: oid('a'),
                partners: vec![
                    RequestPartner { zone: "web".to_string(), tip: oid('b') },
                    RequestPartner { zone: "api".to_string(), tip: oid('c') },
                ],
            },
        ),
    ];
    for (name, base, expected) in cases {
        assert_eq!(recorded_base_of(&base), expected, "case {name}");
    }
}
