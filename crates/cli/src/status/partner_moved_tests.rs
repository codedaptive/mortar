//! partner_moved: the partners whose zone tip is gone or no longer the recorded tip, in record order.

use folders::RequestPartner;
use gitio::Oid;

use super::partner_moved;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn partner(zone: &str, c: char) -> RequestPartner {
    RequestPartner { zone: zone.to_string(), tip: oid(c) }
}

/// One row: its name, the recorded partners, their current tips and the names expected back.
type Case = (&'static str, Vec<RequestPartner>, Vec<Option<Oid>>, Vec<&'static str>);

#[test]
fn partner_moved_cases() {
    let cases: Vec<Case> = vec![
        ("no partners", vec![], vec![], vec![]),
        ("one unmoved", vec![partner("web", 'a')], vec![Some(oid('a'))], vec![]),
        ("one moved", vec![partner("web", 'a')], vec![Some(oid('c'))], vec!["web"]),
        ("one branch gone", vec![partner("web", 'a')], vec![None], vec!["web"]),
        (
            "second of two moved",
            vec![partner("web", 'a'), partner("api", 'b')],
            vec![Some(oid('a')), Some(oid('c'))],
            vec!["api"],
        ),
        (
            "both moved, in record order",
            vec![partner("web", 'a'), partner("api", 'b')],
            vec![None, Some(oid('d'))],
            vec!["web", "api"],
        ),
    ];
    for (name, partners, current, expected) in cases {
        assert_eq!(partner_moved(&partners, &current), expected, "case {name}");
    }
}
