//! Snapshot/is_saved/save -> actual attempt refs, commit trailers and one-branch transactions.
use crate::support::{blob, run, write, Fixture};
use oplog::{oplog_path, read_mortar_trailers, read_op_log, OpPhase, OpResult};
use snapshot::{is_saved, save, snapshot, DEFAULT_SIZE_CAP};
use std::fs;

#[test]
fn attempts_have_independent_snapshot_chains_and_saved_reads_only_the_selected_ref() {
    let f = Fixture::new(&[]);
    let request = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    let request_ref = f.git.read_ref(&f.refname).unwrap();
    let mut target = f.target.clone(); target.attempt = Some(3);
    let selected_ref = "refs/mortar/trysnap/snapshot-zone/request-17/3";
    let mut other = f.target.clone(); other.attempt = Some(4);
    assert!(!is_saved(&f.git, &f.repo.path, &target, &[], DEFAULT_SIZE_CAP).unwrap());
    write(&f.repo.path, "root.txt", b"attempt three content\n");
    let before_index = fs::read(f.common.join("index")).unwrap();
    let first = snapshot(&f.git, &f.repo.path, &target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(first.written);
    assert_eq!(f.git.read_ref(selected_ref).unwrap(), Some(first.commit.clone()));
    assert_eq!(f.git.commit_info(&first.commit).unwrap().parents, vec![f.head.clone()], "request snapshot is not the previous attempt snapshot");
    assert_eq!(blob(&f.git, &first.tree, "root.txt"), b"attempt three content\n");
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), request_ref);
    assert_eq!(f.git.read_ref("refs/mortar/trysnap/snapshot-zone/request-17/4").unwrap(), None);
    assert!(is_saved(&f.git, &f.repo.path, &target, &[], DEFAULT_SIZE_CAP).unwrap());
    assert!(!is_saved(&f.git, &f.repo.path, &other, &[], DEFAULT_SIZE_CAP).unwrap());
    assert!(!is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    let before_log = fs::read(oplog_path(&f.common)).unwrap();
    let repeat = snapshot(&f.git, &f.repo.path, &target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(!repeat.written); assert_eq!(repeat.commit, first.commit);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
    write(&f.repo.path, "root.txt", b"second attempt three content\n");
    let second = snapshot(&f.git, &f.repo.path, &target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert_eq!(f.git.commit_info(&second.commit).unwrap().parents, vec![f.head.clone(), first.commit]);
    assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(request.commit));
    // A None target still writes the original namespace, independently of the attempt's previous snapshot.
    let own = snapshot(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert_eq!(f.git.commit_info(&own.commit).unwrap().parents, vec![f.head.clone(), request_ref.unwrap()]);
    assert_eq!(f.git.read_ref(selected_ref).unwrap(), Some(second.commit));
    assert!(is_saved(&f.git, &f.repo.path, &f.target, &[], DEFAULT_SIZE_CAP).unwrap());
    assert!(is_saved(&f.git, &f.repo.path, &target, &[], DEFAULT_SIZE_CAP).unwrap());
    let records = read_op_log(&f.common).unwrap().records;
    assert!(records.iter().all(|record| record.kind == "snapshot" && record.actor == "snapshot-caller"));
    assert_eq!(records.len(), 8);
    assert_eq!(records[2].moves[0].refname, selected_ref);
    assert_eq!(records[6].moves[0].refname, f.refname);
}

#[test]
fn attempt_saves_move_only_the_try_branch_and_inherit_the_requests_change_id_without_attempt_trailer() {
    for k in [1, 37] {
        let f = Fixture::new(&[]);
        let id = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
        run(&f.git, &["commit", "--allow-empty", "-m", &format!("request seed\n\nMortar-Change-Id: {id}\nMortar-Request: request-17\nMortar-Zone: snapshot-zone\n")]);
        let seed = f.git.rev_parse("HEAD").unwrap().unwrap();
        let request_ref = "refs/heads/req/snapshot-zone/request-17";
        run(&f.git, &["update-ref", request_ref, seed.as_str()]);
        run(&f.git, &["update-ref", &f.refname, seed.as_str()]);
        let branch = format!("try/snapshot-zone/request-17/{k}");
        let branch_ref = format!("refs/heads/{branch}");
        run(&f.git, &["checkout", "-b", &branch]);
        let mut target = f.target.clone(); target.attempt = Some(k);
        write(&f.repo.path, "root.txt", b"attempt saved change\n");
        let saved = save(&f.git, &f.repo.path, &target, "Save attempt", &[], DEFAULT_SIZE_CAP).unwrap();
        assert!(saved.written);
        assert_eq!(saved.change_id.as_ref().unwrap().as_str(), id);
        assert_eq!(f.git.read_ref(&branch_ref).unwrap(), Some(saved.commit.clone()));
        assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(saved.commit.clone()));
        assert_eq!(f.git.read_ref(request_ref).unwrap(), Some(seed.clone()));
        assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(seed.clone()));
        let info = f.git.commit_info(&saved.commit).unwrap();
        assert_eq!(info.parents, vec![seed]);
        assert_eq!(blob(&f.git, &info.tree, "root.txt"), b"attempt saved change\n");
        assert_eq!(info.message, format!("Save attempt\n\nMortar-Change-Id: {id}\nMortar-Request: request-17\nMortar-Zone: snapshot-zone\n"));
        assert_eq!(read_mortar_trailers(&f.git, &info.message).unwrap().change_id, saved.change_id);
        let snapshot_ref = format!("refs/mortar/trysnap/snapshot-zone/request-17/{k}");
        let snap = f.git.read_ref(&snapshot_ref).unwrap().unwrap();
        assert_eq!(f.git.commit_info(&snap).unwrap().tree, saved.tree);
        assert!(is_saved(&f.git, &f.repo.path, &target, &[], DEFAULT_SIZE_CAP).unwrap());
        assert!(run(&f.git, &["status", "--porcelain", "-z"]).is_empty());
        let records = read_op_log(&f.common).unwrap().records;
        assert_eq!(records.len(), 4);
        assert_eq!(records[0].moves[0].refname, snapshot_ref);
        assert_eq!(records[2].kind, "save");
        assert_eq!(records[2].moves[0].refname, branch_ref);
        assert_eq!(records[3].phase, OpPhase::Done); assert_eq!(records[3].result, Some(OpResult::Moved));
        let before_log = fs::read(oplog_path(&f.common)).unwrap();
        let repeat = save(&f.git, &f.repo.path, &target, "No change", &[], DEFAULT_SIZE_CAP).unwrap();
        assert!(!repeat.written); assert_eq!(repeat.commit, saved.commit);
        assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), before_log);
    }
}
