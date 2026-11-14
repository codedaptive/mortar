//! decode_folder_record: the round trip over many records, free key order and whitespace, and each refusal
//! with the key or value it names.

use gitio::Oid;

use super::{decode_folder_record, encode_folder_record, FolderKind, FolderRecord, RecordError};

const BASE: &str = "0123456789abcdef0123456789abcdef01234567";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn records() -> Vec<FolderRecord> {
    vec![
        FolderRecord {
            kind: FolderKind::Zone,
            zone: "folders".to_string(),
            request: None,
            base: oid(BASE),
            work_order_paths: Vec::new(),
            needed_paths: Vec::new(),
            parent: None,
        },
        FolderRecord {
            kind: FolderKind::Request,
            zone: "join".to_string(),
            request: Some("WO-join-7".to_string()),
            base: oid("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            work_order_paths: strings(&["crates/join/src", "crates/join/tests", "Cargo.lock"]),
            needed_paths: strings(&["crates/gitio"]),
            parent: Some("workcrews/join".to_string()),
        },
        FolderRecord {
            kind: FolderKind::Request,
            zone: "q\"uote".to_string(),
            request: Some("back\\slash\\".to_string()),
            base: oid(BASE),
            work_order_paths: strings(&["new\nline", "tab\tand\r\u{8}\u{c}", "\u{0}\u{1f}", ""]),
            needed_paths: strings(&["é主😀\u{2028}", "/slash"]),
            parent: Some("refs/heads/ü\"\\".to_string()),
        },
        FolderRecord {
            kind: FolderKind::Zone,
            zone: String::new(),
            request: None,
            base: oid(BASE),
            work_order_paths: strings(&["a"]),
            needed_paths: strings(&["b", "c"]),
            parent: Some("main".to_string()),
        },
    ]
}

fn refused(text: &str) -> String {
    match decode_folder_record(text) {
        Err(RecordError { reason }) => reason,
        Ok(r) => panic!("{text:?} decoded to {r:?}"),
    }
}

/// The worked zone record's text with `key`'s value replaced by `value`.
fn zone_with(key: &str, value: &str) -> String {
    let mut members = vec![
        ("kind", "\"zone\""),
        ("zone", "\"folders\""),
        ("request", "null"),
        ("base", "\"0123456789abcdef0123456789abcdef01234567\""),
        ("work_order_paths", "[]"),
        ("needed_paths", "[]"),
        ("parent", "null"),
    ];
    for m in &mut members {
        if m.0 == key {
            m.1 = value;
        }
    }
    let body: Vec<String> = members.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
    format!("{{{}}}", body.join(","))
}

#[test]
fn round_trip_returns_each_record() {
    for r in records() {
        assert_eq!(decode_folder_record(&encode_folder_record(&r)), Ok(r));
    }
}

#[test]
fn key_order_is_free_and_whitespace_is_allowed() {
    let text = " {\n \"parent\" : \"main\" ,\t\"needed_paths\":[ \"b\" , \"c\" ], \"work_order_paths\" :[\"a\"],\r\n\
                \"base\":\"0123456789abcdef0123456789abcdef01234567\", \"request\": null, \"zone\":\"\",\"kind\":\"zone\" } \n";
    assert_eq!(decode_folder_record(text), Ok(records().swap_remove(3)));
}

#[test]
fn zone_with_is_the_zone_record() {
    assert_eq!(decode_folder_record(&zone_with("", "")), Ok(records().swap_remove(0)));
}

#[test]
fn not_an_object_is_refused() {
    assert_eq!(
        refused("[]"),
        "unexpected '[' at byte 0 where the record's '{' belongs: the record is not a JSON object"
    );
}

#[test]
fn missing_key_is_refused_by_name() {
    let text = zone_with("", "").replace(",\"needed_paths\":[]", "");
    assert_eq!(refused(&text), "missing key \"needed_paths\"");
    let text = zone_with("", "").replace("\"kind\":\"zone\",", "");
    assert_eq!(refused(&text), "missing key \"kind\"");
}

#[test]
fn unknown_key_is_refused_by_name() {
    let text = zone_with("", "").replace("\"parent\":null", "\"parent\":null,\"owner\":\"simms\"");
    assert_eq!(refused(&text), "unknown key \"owner\"");
}

#[test]
fn repeated_key_is_refused_by_name() {
    let text = zone_with("", "").replace("\"zone\":\"folders\"", "\"zone\":\"folders\",\"zone\":\"join\"");
    assert_eq!(refused(&text), "key \"zone\" appears twice");
}

#[test]
fn value_of_the_wrong_type_is_refused_naming_the_key() {
    assert_eq!(refused(&zone_with("kind", "null")), "kind is not a string");
    assert_eq!(refused(&zone_with("zone", "[\"folders\"]")), "zone is not a string");
    assert_eq!(refused(&zone_with("base", "null")), "base is not a string");
    assert_eq!(refused(&zone_with("request", "[]")), "request is not a string or null");
    assert_eq!(refused(&zone_with("parent", "[\"main\"]")), "parent is not a string or null");
    assert_eq!(refused(&zone_with("work_order_paths", "\"a\"")), "work_order_paths is not an array of strings");
    assert_eq!(refused(&zone_with("needed_paths", "null")), "needed_paths is not an array of strings");
    assert_eq!(
        refused(&zone_with("zone", "7")),
        "unexpected '7' at byte 22 as the value of \"zone\", which must be a string, null or an array of strings"
    );
}

#[test]
fn unknown_kind_is_refused_naming_it() {
    assert_eq!(refused(&zone_with("kind", "\"Zone\"")), "kind \"Zone\" is neither \"zone\" nor \"request\"");
}

#[test]
fn base_that_is_not_an_oid_is_refused_naming_it() {
    assert_eq!(refused(&zone_with("base", "\"HEAD\"")), "base \"HEAD\" is not a git object id");
    assert_eq!(
        refused(&zone_with("base", "\"0123456789ABCDEF0123456789abcdef01234567\"")),
        "base \"0123456789ABCDEF0123456789abcdef01234567\" is not a git object id"
    );
}

#[test]
fn zone_record_with_a_request_is_refused() {
    assert_eq!(refused(&zone_with("request", "\"WO-1\"")), "a zone record has request \"WO-1\"; it must be null");
}

#[test]
fn request_record_without_a_request_is_refused() {
    assert_eq!(
        refused(&zone_with("kind", "\"request\"")),
        "a request record has request null; it must name the request"
    );
}

#[test]
fn trailing_text_is_refused() {
    let text = format!("{}\n{}", zone_with("", ""), zone_with("", ""));
    assert_eq!(refused(&text), "unexpected '{' at byte 152 after the record");
}

#[test]
fn every_truncation_of_a_record_says_the_text_ends_early() {
    for r in records() {
        let text = encode_folder_record(&r);
        for (cut, _) in text.char_indices() {
            assert_eq!(
                refused(&text[..cut]),
                "the text ends before the record is complete",
                "{:?}",
                &text[..cut]
            );
        }
    }
}
