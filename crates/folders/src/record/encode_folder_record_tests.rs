//! encode_folder_record: literal lines written by hand for each kind, escapes, and no newline.

use gitio::Oid;

use super::{encode_folder_record, FolderKind, FolderRecord};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn request_record_worked_example() {
    let r = FolderRecord {
        kind: FolderKind::Request,
        zone: "folders".to_string(),
        request: Some("rf1".to_string()),
        base: oid("0123456789abcdef0123456789abcdef01234567"),
        work_order_paths: vec!["crates/folders/src/record".to_string(), "crates/folders/src/plan".to_string()],
        needed_paths: vec!["crates/gitio".to_string()],
        parent: Some("workcrews/folders".to_string()),
    };
    assert_eq!(
        encode_folder_record(&r),
        r#"{"kind":"request","zone":"folders","request":"rf1","base":"0123456789abcdef0123456789abcdef01234567","work_order_paths":["crates/folders/src/record","crates/folders/src/plan"],"needed_paths":["crates/gitio"],"parent":"workcrews/folders"}"#
    );
}

#[test]
fn zone_record_writes_null_request_and_parent_and_empty_lists() {
    let r = FolderRecord {
        kind: FolderKind::Zone,
        zone: "join".to_string(),
        request: None,
        base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: None,
    };
    assert_eq!(
        encode_folder_record(&r),
        r#"{"kind":"zone","zone":"join","request":null,"base":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work_order_paths":[],"needed_paths":[],"parent":null}"#
    );
}

#[test]
fn strings_are_escaped_and_the_line_has_no_newline() {
    let r = FolderRecord {
        kind: FolderKind::Request,
        zone: "a\"b".to_string(),
        request: Some("back\\slash".to_string()),
        base: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        work_order_paths: vec!["line\nbreak".to_string(), "tab\there".to_string()],
        needed_paths: vec!["\u{1}é主😀".to_string()],
        parent: Some("refs/heads/ü".to_string()),
    };
    let line = encode_folder_record(&r);
    assert_eq!(
        line,
        "{\"kind\":\"request\",\"zone\":\"a\\\"b\",\"request\":\"back\\\\slash\",\
         \"base\":\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\",\
         \"work_order_paths\":[\"line\\nbreak\",\"tab\\there\"],\
         \"needed_paths\":[\"\\u0001é主😀\"],\"parent\":\"refs/heads/ü\"}"
    );
    assert!(!line.contains('\n'));
}

#[test]
fn attempt_record_writes_its_number_right_after_kind() {
    let r = FolderRecord {
        kind: FolderKind::Attempt(3),
        zone: "folders".to_string(),
        request: Some("rf6".to_string()),
        base: oid("0123456789abcdef0123456789abcdef01234567"),
        work_order_paths: vec!["crates/folders/src/record".to_string()],
        needed_paths: Vec::new(),
        parent: Some("req/folders/rf6".to_string()),
    };
    assert_eq!(
        encode_folder_record(&r),
        r#"{"kind":"attempt","attempt":"3","zone":"folders","request":"rf6","base":"0123456789abcdef0123456789abcdef01234567","work_order_paths":["crates/folders/src/record"],"needed_paths":[],"parent":"req/folders/rf6"}"#
    );
}
