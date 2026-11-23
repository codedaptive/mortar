use std::fs;
use crate::support::{run, setup, write, Fixture};
use folders::{make_request_folder, make_request_folder_on, make_zone_folder, need, read_recorded_base, rewrite_request_base, FolderError, FolderKind, FolderRecord, RecordedBase, RequestPartner};
use folders::make::read_folder_record;
use folders::record::record_path;
use gitio::{Git, Oid};
use oplog::{oplog_path, KillPidProbe};

fn oid_output(git: &Git, args: &[&str]) -> Oid {
    Oid::parse(std::str::from_utf8(&run(git, args)).unwrap().trim()).unwrap()
}

fn merges(f: &Fixture) -> (RecordedBase, RecordedBase) {
    run(&f.git, &["update-ref", "refs/heads/zone/z", f.head.as_str()]);
    write(&f.repo.path, "other/file", b"partner one\n");
    run(&f.git, &["add", "other/file"]);
    run(&f.git, &["commit", "-m", "partner one"]);
    let p1 = oid_output(&f.git, &["rev-parse", "HEAD"]);
    run(&f.git, &["update-ref", "refs/heads/zone/p", p1.as_str()]);
    let tree1 = oid_output(&f.git, &["rev-parse", "HEAD^{tree}"]);
    let m1 = oid_output(&f.git, &["commit-tree", tree1.as_str(), "-p", f.head.as_str(), "-p", p1.as_str(), "-m", "merge one"]);
    write(&f.repo.path, "other/file", b"partner two\n");
    run(&f.git, &["add", "other/file"]);
    run(&f.git, &["commit", "-m", "partner two"]);
    let p2 = oid_output(&f.git, &["rev-parse", "HEAD"]);
    let tree2 = oid_output(&f.git, &["rev-parse", "HEAD^{tree}"]);
    let m2 = oid_output(&f.git, &["commit-tree", tree2.as_str(), "-p", f.head.as_str(), "-p", p2.as_str(), "-m", "merge two"]);
    (RecordedBase { base: m1, partners: vec![RequestPartner { zone: "p".into(), tip: p1 }] },
     RecordedBase { base: m2, partners: vec![RequestPartner { zone: "p".into(), tip: p2 }] })
}

fn expected(base: Oid) -> FolderRecord {
    FolderRecord { kind: FolderKind::Request, zone: "z".into(), request: Some("r".into()), base,
        work_order_paths: vec!["kit/zone".into()], needed_paths: vec![], parent: Some("zone/z".into()) }
}

fn record_text(base: &Oid, partners: &str) -> String {
    format!("{{\"kind\":\"request\",\"zone\":\"z\",\"request\":\"r\",\"base\":\"{base}\",\"work_order_paths\":[\"kit/zone\"],\"needed_paths\":[],\"parent\":\"zone/z\"{partners}}}\n")
}

#[test]
fn make_request_folder_on_cases() {
    for with_partner in [false, true] {
        let f = Fixture::new();
        let (merge, _) = merges(&f);
        let next_tip = merge.partners[0].tip.clone();
        let base = if with_partner { merge } else { RecordedBase { base: f.head.clone(), partners: vec![] } };
        let path = f.repo.path.join("request");
        let s = setup(&f.git, &path);
        let paths = ["kit/zone".into()];
        let record = make_request_folder_on(&s, "z", "r", &paths, &base).unwrap();
        assert_eq!(record, expected(base.base.clone()));
        let branch = "refs/heads/req/z/r";
        assert_eq!(f.git.read_ref(branch).unwrap(), Some(base.base.clone()));
        let local = Git { repo: path.clone(), ..f.git.clone() };
        assert_eq!(local.rev_parse("HEAD").unwrap(), Some(base.base.clone()));
        let dir = f.git.git_dir(&path).unwrap();
        assert_eq!(read_recorded_base(&dir).unwrap(), Some(base.clone()));
        assert_eq!(read_folder_record(&dir).unwrap(), Some(record.clone()));
        let suffix = if with_partner { format!(",\"partners\":[\"p:{}\"]", base.partners[0].tip) } else { String::new() };
        assert_eq!(fs::read_to_string(record_path(&dir)).unwrap(), record_text(&base.base, &suffix));
        let bytes = fs::read(record_path(&dir)).unwrap();
        let log = fs::read(oplog_path(&f.common)).unwrap();
        run(&local, &["sparse-checkout", "set", "other"]);
        assert!(!path.join("kit/zone/file").exists());
        assert_eq!(make_request_folder_on(&s, "z", "r", &paths, &base).unwrap(), record);
        assert_eq!(fs::read(path.join("kit/zone/file")).unwrap(), b"zone\n");
        assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log);
        assert_eq!(f.git.read_ref(branch).unwrap(), Some(base.base.clone()));
        if with_partner {
            let other_base = RecordedBase { base: f.head.clone(), partners: base.partners.clone() };
            let other_partners = RecordedBase { base: base.base.clone(), partners: vec![] };
            let plain_tip = RecordedBase { base: f.head.clone(), partners: vec![] };
            for (name, wanted) in [("base", other_base), ("partners", other_partners), ("plain tip", plain_tip)] {
                let error = make_request_folder_on(&s, "z", "r", &paths, &wanted).unwrap_err();
                match (name, error) {
                    ("base" | "plain tip", FolderError::BranchExists { branch }) => assert_eq!(branch, "req/z/r"),
                    ("partners", FolderError::DifferentFolder { path: actual, existing }) => { assert_eq!(actual, path); assert_eq!(existing, record); },
                    (_, error) => panic!("{name}: {error:?}"),
                }
                assert_eq!(fs::read(record_path(&dir)).unwrap(), bytes);
                assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log);
                assert_eq!(f.git.read_ref(branch).unwrap(), Some(base.base.clone()));
            }
        } else {
            run(&f.git, &["update-ref", "refs/heads/zone/z", next_tip.as_str()]);
            run(&local, &["sparse-checkout", "set", "other"]);
            assert!(!path.join("kit/zone/file").exists());
            let new_tip = RecordedBase { base: next_tip, partners: vec![] };
            assert_eq!(make_request_folder_on(&s, "z", "r", &paths, &new_tip).unwrap(), record);
            assert_eq!(fs::read(path.join("kit/zone/file")).unwrap(), b"zone\n");
            assert_eq!(read_recorded_base(&dir).unwrap(), Some(base.clone()));
            assert_eq!(fs::read(record_path(&dir)).unwrap(), bytes);
            assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log);
            assert_eq!(f.git.read_ref(branch).unwrap(), Some(base.base.clone()));
        }
    }
}

#[test]
fn read_recorded_base_reads_old_record() {
    let f = Fixture::new();
    run(&f.git, &["update-ref", "refs/heads/zone/z", f.head.as_str()]);
    let path = f.repo.path.join("request");
    make_request_folder(&setup(&f.git, &path), "z", "r", &["kit/zone".into()]).unwrap();
    let dir = f.git.git_dir(&path).unwrap();
    fs::write(record_path(&dir), format!("{{\"kind\":\"request\",\"zone\":\"z\",\"request\":\"r\",\"base\":\"{}\",\"work_order_paths\":[\"kit/zone\"],\"needed_paths\":[],\"parent\":\"zone/z\"}}\n", f.head)).unwrap();
    assert_eq!(read_recorded_base(&dir).unwrap(), Some(RecordedBase { base: f.head.clone(), partners: vec![] }));
    assert_eq!(read_folder_record(&dir).unwrap(), Some(expected(f.head.clone())));
}

#[test]
fn rewrite_request_base_cases() {
    let f = Fixture::new();
    let (first, second) = merges(&f);
    let path = f.repo.path.join("request");
    let old = make_request_folder_on(&setup(&f.git, &path), "z", "r", &["kit/zone".into()], &first).unwrap();
    let dir = f.git.git_dir(&path).unwrap();
    for base in [second.clone(), RecordedBase { base: f.head.clone(), partners: vec![] }] {
        let mut expected = old.clone();
        expected.base = base.base.clone();
        assert_eq!(rewrite_request_base(&f.git, &path, &base).unwrap(), expected);
        let suffix = if base.partners.is_empty() { String::new() } else { format!(",\"partners\":[\"p:{}\"]", base.partners[0].tip) };
        assert_eq!(fs::read_to_string(record_path(&dir)).unwrap(), record_text(&base.base, &suffix));
        assert_eq!(read_recorded_base(&dir).unwrap(), Some(base));
        assert_eq!(f.git.read_ref("refs/heads/req/z/r").unwrap(), Some(first.base.clone()));
    }
    let zone_path = f.repo.path.join("zone");
    make_zone_folder(&setup(&f.git, &zone_path), "z", &["kit/zone".into()], Some(&f.head)).unwrap();
    let zone_file = record_path(&f.git.git_dir(&zone_path).unwrap());
    let before = fs::read(&zone_file).unwrap();
    assert!(matches!(rewrite_request_base(&f.git, &zone_path, &second), Err(FolderError::ZoneFolder { .. })));
    assert_eq!(fs::read(zone_file).unwrap(), before);
}

#[test]
fn need_preserves_recorded_partners() {
    let f = Fixture::new();
    let (base, _) = merges(&f);
    let path = f.repo.path.join("request");
    make_request_folder_on(&setup(&f.git, &path), "z", "r", &["kit/zone".into()], &base).unwrap();
    let result = need(&f.git, &path, "needed", &KillPidProbe).unwrap();
    assert_eq!(result.needed_paths, ["needed"]);
    assert_eq!(fs::read(path.join("needed/file")).unwrap(), b"needed\n");
    assert_eq!(read_recorded_base(&f.git.git_dir(&path).unwrap()).unwrap(), Some(base));
}
