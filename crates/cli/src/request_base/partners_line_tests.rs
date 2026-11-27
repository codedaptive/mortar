//! partners_line: the partners line with full tips in order, and "partners: none" with none.

use folders::RequestPartner;
use gitio::Oid;

use super::partners_line;

fn partner(zone: &str, c: char) -> RequestPartner {
    RequestPartner { zone: zone.to_string(), tip: Oid::parse(&c.to_string().repeat(40)).expect("a valid oid") }
}

#[test]
fn partners_line_cases() {
    let cases: Vec<(&str, Vec<RequestPartner>, &str)> = vec![
        ("none", vec![], "partners: none"),
        ("one", vec![partner("web", 'a')], "partners: web at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        (
            "two in order",
            vec![partner("web", 'a'), partner("api", 'b')],
            "partners: web at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, api at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
    ];
    for (name, partners, expected) in cases {
        assert_eq!(partners_line(&partners), expected, "case {name}");
    }
}
