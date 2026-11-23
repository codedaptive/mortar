use gitio::Oid;
use super::{FolderKind, FolderRecord, RequestPartner};
use super::decode_record_and_partners::decode_record_and_partners;
use super::encode_record_and_partners::encode_record_and_partners;

#[test]
fn decode_record_and_partners_cases() {
    let record = FolderRecord {
        kind: FolderKind::Request, zone: "z".into(), request: Some("r".into()),
        base: Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
        work_order_paths: vec!["src".into()], needed_paths: vec![], parent: Some("zone/z".into()),
    };
    let p = RequestPartner { zone: "p".into(), tip: Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() };
    let q = RequestPartner { zone: "q".into(), tip: Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap() };
    let plain = r#"{"kind":"request","zone":"z","request":"r","base":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work_order_paths":["src"],"needed_paths":[],"parent":"zone/z"}"#;
    let one = r#"{"kind":"request","zone":"z","request":"r","base":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work_order_paths":["src"],"needed_paths":[],"parent":"zone/z","partners":["p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"]}"#;
    let two = r#"{"kind":"request","zone":"z","request":"r","base":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work_order_paths":["src"],"needed_paths":[],"parent":"zone/z","partners":["p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","q:cccccccccccccccccccccccccccccccccccccccc"]}"#;
    for (name, text, partners) in [("none", plain, vec![]), ("one", one, vec![p.clone()]), ("two", two, vec![p.clone(), q.clone()])] {
        assert_eq!(decode_record_and_partners(text), Ok((record.clone(), partners)), "{name}");
    }
    let zone = one.replace("\"kind\":\"request\"", "\"kind\":\"zone\"").replace("\"request\":\"r\"", "\"request\":null");
    let attempt = one.replace("\"kind\":\"request\"", "\"kind\":\"attempt\",\"attempt\":\"1\"");
    let empty = one.replace("[\"p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"]", "[]");
    let non_array = one.replace("[\"p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"]", "null");
    let duplicate = one.replace("\"partners\":[", "\"partners\":[],\"partners\":[");
    let missing_colon = one.replace("p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "pbbbb");
    let bad_zone = one.replace("p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "a b:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let dot_zone = one.replace("p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", ".x:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let bad_oid = one.replace("p:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "p:nope");
    for (text, token) in [
        (zone, "\"partners\" key on a zone record"), (attempt, "\"partners\" key on an attempt record"),
        (empty, "partners is empty"), (non_array, "partners is not an array of strings"), (duplicate, "appears twice"),
        (missing_colon, "partner \"pbbbb\""), (bad_zone, "partner \"a b:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\""),
        (dot_zone, "partner \".x:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\""), (bad_oid, "partner \"p:nope\""),
    ] {
        let error = decode_record_and_partners(&text).unwrap_err();
        assert!(error.reason.contains(token), "{text}: {:?}", error.reason);
    }
    for partners in [vec![], vec![p.clone()], vec![q.clone(), p.clone()]] {
        assert_eq!(decode_record_and_partners(&encode_record_and_partners(&record, &partners)), Ok((record.clone(), partners)));
    }
}
