//! partners_json: each RequestPartner as one encoded JSON object, in order, with the zone escaped.

use folders::RequestPartner;
use gitio::Oid;

use super::partners_json;

fn partner(zone: &str, c: char) -> RequestPartner {
    RequestPartner { zone: zone.to_string(), tip: Oid::parse(&c.to_string().repeat(40)).expect("a valid oid") }
}

#[test]
fn partners_json_cases() {
    let cases: Vec<(&str, Vec<RequestPartner>, Vec<&str>)> = vec![
        ("none", vec![], vec![]),
        (
            "two in order",
            vec![partner("web", 'a'), partner("api", 'b')],
            vec![
                r#"{"zone":"web","tip":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
                r#"{"zone":"api","tip":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
            ],
        ),
        (
            "zone with a quote and a backslash",
            vec![partner(r#"we"b\x"#, 'c')],
            vec![r#"{"zone":"we\"b\\x","tip":"cccccccccccccccccccccccccccccccccccccccc"}"#],
        ),
    ];
    for (name, partners, expected) in cases {
        assert_eq!(partners_json(&partners), expected, "case {name}");
    }
}
