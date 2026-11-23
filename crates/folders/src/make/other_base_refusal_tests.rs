use std::path::Path;
use gitio::Oid;
use super::{FolderError, other_base_refusal::other_base_refusal};
use crate::record::{FolderKind, FolderRecord, RecordedBase, RequestPartner};

#[test]
fn other_base_refusal_cases() {
    let existing = FolderRecord {
        kind: FolderKind::Request, zone: "z".into(), request: Some("r".into()),
        base: Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
        work_order_paths: vec!["src".into()], needed_paths: vec![], parent: Some("zone/z".into()),
    };
    let p = RequestPartner { zone: "p".into(), tip: Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() };
    let q = RequestPartner { zone: "q".into(), tip: Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap() };
    let changed_p = RequestPartner { zone: "p".into(), tip: Oid::parse("dddddddddddddddddddddddddddddddddddddddd").unwrap() };
    let path = Path::new("/tmp/request-folder");
    let partners = vec![p.clone(), q.clone()];
    for (name, base, wanted_partners, expected) in [
        ("same", existing.base.clone(), partners.clone(), "none"),
        ("base", Oid::parse("dddddddddddddddddddddddddddddddddddddddd").unwrap(), partners.clone(), "branch"),
        ("tip", existing.base.clone(), vec![changed_p, q.clone()], "folder"),
        ("order", existing.base.clone(), vec![q.clone(), p.clone()], "folder"),
        ("missing", existing.base.clone(), vec![p.clone()], "folder"),
    ] {
        let wanted = RecordedBase { base, partners: wanted_partners };
        let result = other_base_refusal(&wanted, &existing, &partners, "req/z/r", path);
        match (expected, result) {
            ("none", None) => (),
            ("branch", Some(FolderError::BranchExists { branch })) => assert_eq!(branch, "req/z/r", "{name}"),
            ("folder", Some(FolderError::DifferentFolder { path: actual, existing: record })) => {
                assert_eq!(actual, path, "{name}"); assert_eq!(record, existing, "{name}");
            }
            (_, other) => panic!("{name}: unexpected refusal {other:?}"),
        }
    }
}
