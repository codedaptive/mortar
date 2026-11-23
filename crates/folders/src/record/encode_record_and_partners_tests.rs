use gitio::Oid;
use super::{FolderKind, FolderRecord, RequestPartner};
use super::encode_record_and_partners::encode_record_and_partners;

#[test]
fn encode_record_and_partners_cases() {
    let record = FolderRecord {
        kind: FolderKind::Request, zone: "z".into(), request: Some("r".into()),
        base: Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
        work_order_paths: vec!["src".into()], needed_paths: vec![], parent: Some("zone/z".into()),
    };
    let p = RequestPartner { zone: "p".into(), tip: Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() };
    let q = RequestPartner { zone: "q".into(), tip: Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap() };
    for (name, partners, expected) in [
        ("none", vec![], r#"{"kind":"request","zone":"z","request":"r","base":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work_order_paths":["src"],"needed_paths":[],"parent":"zone/z"}"#),
        ("one", vec![p.clone()], r#"{"kind":"request","zone":"z","request":"r","base":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work_order_paths":["src"],"needed_paths":[],"parent":"zone/z","partners":["p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"]}"#),
        ("two", vec![p.clone(), q.clone()], r#"{"kind":"request","zone":"z","request":"r","base":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work_order_paths":["src"],"needed_paths":[],"parent":"zone/z","partners":["p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","q:cccccccccccccccccccccccccccccccccccccccc"]}"#),
    ] {
        assert_eq!(encode_record_and_partners(&record, &partners), expected, "{name}");
    }
}
