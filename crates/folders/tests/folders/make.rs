use crate::support::{assert_branch_op, assert_ordinary, run, setup, write, Fixture};
use folders::{make_request_folder, make_zone_folder, FolderError, FolderKind, FolderRecord};
use folders::make::read_folder_record;
use folders::record::record_path;
use gitio::Git;
use oplog::{oplog_path, read_op_log};
use std::fs;

#[test]
fn missing_zone_and_request_branches_are_logged_and_folders_have_exact_records_and_cones() {
    // Joins make_zone_folder/make_request_folder -> gitio worktrees/sparse paths -> oplog branch moves and records.
    let f = Fixture::new();
    let zone_path = f.repo.path.join("zone-folder");
    let paths = ["kit/zone/".into(), "kit/dependency".into(), "kit/zone".into()];
    let zone = make_zone_folder(&setup(&f.git, &zone_path), "red", &paths, Some(&f.head)).unwrap();
    let expected = FolderRecord {
        kind: FolderKind::Zone, zone: "red".into(), request: None, base: f.head.clone(),
        work_order_paths: vec!["kit/dependency".into(), "kit/zone".into()], needed_paths: vec![], parent: None,
    };
    assert_eq!(zone, expected);
    assert_ordinary(&f.git, &zone_path, "zone/red", &f.head, &["kit/dependency", "kit/zone"]);
    assert_eq!(fs::read(zone_path.join("kit/dependency/file")).unwrap(), b"dependency\n");
    assert_eq!(fs::read(zone_path.join("kit/zone/file")).unwrap(), b"zone\n");
    assert!(!zone_path.join("needed").exists());
    let zone_dir = f.git.git_dir(&zone_path).unwrap();
    assert_eq!(record_path(&zone_dir), zone_dir.join("mortar-folder.json"));
    assert_eq!(read_folder_record(&zone_dir).unwrap(), Some(expected));
    assert!(fs::read(record_path(&zone_dir)).unwrap().ends_with(b"\n"));
    assert!(!zone_dir.join("mortar-folder.json.tmp").exists());
    let req_path = f.repo.path.join("request-folder");
    let request = make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();
    let expected = FolderRecord {
        kind: FolderKind::Request, zone: "red".into(), request: Some("one".into()), base: f.head.clone(),
        work_order_paths: vec!["kit/zone".into()], needed_paths: vec![], parent: Some("zone/red".into()),
    };
    assert_eq!(request, expected);
    assert_ordinary(&f.git, &req_path, "req/red/one", &f.head, &["kit/zone"]);
    assert!(!req_path.join("kit/dependency").exists());
    assert_eq!(read_folder_record(&f.git.git_dir(&req_path).unwrap()).unwrap(), Some(expected));
    assert_eq!(f.git.read_ref("refs/heads/zone/red").unwrap(), Some(f.head.clone()));
    assert_eq!(f.git.read_ref("refs/heads/req/red/one").unwrap(), Some(f.head.clone()));
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 4);
    assert_branch_op(&log.records[..2], "zone/red", &f.head);
    assert_branch_op(&log.records[2..], "req/red/one", &f.head);
    assert!(!f.common.join("mortar/locks/folders.lock").exists());
}

#[test]
fn existing_zone_uses_current_tip_ignores_start_and_request_is_cut_from_newest_zone_commit() {
    // Joins existing-zone selection -> ordinary branch checkout -> request's logged expected-absent creation.
    let f = Fixture::new();
    write(&f.repo.path, "kit/zone/file", b"new zone tip\n");
    run(&f.git, &["add", "kit/zone/file"]);
    run(&f.git, &["commit", "-m", "new tip"]);
    let tip = f.git.rev_parse("HEAD").unwrap().unwrap();
    assert_ne!(tip, f.head);
    run(&f.git, &["update-ref", "refs/heads/zone/red", tip.as_str()]);
    let zone_path = f.repo.path.join("zone-folder");
    let record = make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
    assert_eq!(record.base, tip);
    assert_ordinary(&f.git, &zone_path, "zone/red", &tip, &["kit/zone"]);
    assert!(!oplog_path(&f.common).exists());
    let zone_git = Git { repo: zone_path.clone(), ..f.git.clone() };
    write(&zone_path, "kit/zone/file", b"newest zone tip\n");
    run(&zone_git, &["add", "kit/zone/file"]);
    run(&zone_git, &["commit", "-m", "zone advances"]);
    let newest = zone_git.rev_parse("HEAD").unwrap().unwrap();
    assert_ne!(newest, tip);
    let req_path = f.repo.path.join("request-folder");
    let record = make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();
    assert_eq!(record.base, newest);
    assert_ordinary(&f.git, &req_path, "req/red/one", &newest, &["kit/zone"]);
    assert_eq!(fs::read(req_path.join("kit/zone/file")).unwrap(), b"newest zone tip\n");
    let log = read_op_log(&f.common).unwrap();
    assert_branch_op(&log.records, "req/red/one", &newest);
}

#[test]
fn missing_zone_without_start_refuses_both_make_variants_without_branches_or_worktrees() {
    // Joins zone-start refusal -> lock release -> absence of real ref/worktree/log side effects.
    let f = Fixture::new();
    let path = f.repo.path.join("not-made");
    let s = setup(&f.git, &path);
    let first = make_zone_folder(&s, "absent", &["kit/zone".into()], None).unwrap_err();
    let second = make_request_folder(&s, "absent", "one", &["kit/zone".into()]).unwrap_err();
    for error in [first, second] {
        assert!(error.to_string().contains("zone/absent"));
        assert!(matches!(error, FolderError::ZoneBranchMissing { branch } if branch == "zone/absent"));
    }
    assert!(!path.exists());
    assert_eq!(f.git.read_ref("refs/heads/zone/absent").unwrap(), None);
    assert_eq!(f.git.read_ref("refs/heads/req/absent/one").unwrap(), None);
    assert!(!oplog_path(&f.common).exists());
    assert_eq!(f.git.worktree_list().unwrap().len(), 1);
    assert!(!f.common.join("mortar/locks/folders.lock").exists());
}
