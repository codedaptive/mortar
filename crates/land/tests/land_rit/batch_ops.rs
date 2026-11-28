//! Public Land batch/commit_files -> real Gitio/Snapshot/Oplog: moves, actor, phase, keys and change identities.
use crate::{batch_fixture::{Batch, ACTOR, IDS, INTEGRATION, MAIN}, owed_ops::{locked, raw, refs}};
use gitio::{Oid, RunOpts};
use land::{BatchBackOutAnswer, CommitFilesAnswer, PublishAnswer};
use oplog::{ChangeId, OpPhase, OpRecord, OpResult, RefMove};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

fn records(b: &Batch, kind: &str) -> Vec<OpRecord> {
    oplog::read_op_log(&b.f.common).unwrap().records.into_iter().filter(|r| r.kind == kind).collect()
}
fn assert_pair(pair: &[OpRecord], kind: &str, refname: &str, old: &Oid, new: &Oid, cid: &str) {
    assert_eq!(pair.len(), 2); assert!(!pair[0].op_id.is_empty()); assert_eq!(pair[0].op_id, pair[1].op_id);
    let key = if kind == "publish-push" { format!("publish-push:origin:{refname}:{old}:{new}") }
        else { format!("{kind}:{refname}:{old}:{new}:{}", pair[0].op_id) };
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    for (r, (phase, result)) in pair.iter().zip([(OpPhase::Intent, None), (OpPhase::Done, Some(OpResult::Moved))]) {
        assert!(r.at > 0 && r.at <= now); // Real metadata, not a duration or speed assertion.
        assert_eq!(*r, OpRecord {
            op_id: pair[0].op_id.clone(), key: key.clone(), kind: kind.into(), actor: ACTOR.into(),
            reason: format!("{kind} {cid} moves {refname} from {old} to {new}"), at: r.at, phase,
            moves: vec![RefMove { refname: refname.into(), old: Some(old.clone()), new: new.clone() }], result,
        });
    }
}

#[test]
fn rit_landing_and_backing_out_one_or_two_members_log_the_tip_and_newest_change_id() {
    for count in [1, 2] {
        let b = Batch::new("second", "second"); for i in 0..count { b.land(i); }
        let lands = records(&b, "land"); assert_eq!(lands.len(), count * 2);
        for i in 0..count {
            let from = if i == 0 { &b.origin } else { &b.landed[i - 1] };
            assert_pair(&lands[i * 2..i * 2 + 2], "land", INTEGRATION, from, &b.landed[i], IDS[i]);
        }
        let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert_eq!(locked(&b.f, || land::back_out_batch(&b.f.git, &refs(), ACTOR)), BatchBackOutAnswer::BackedOut {
            change_id: ChangeId::parse(IDS[count - 1]).unwrap(),
            change_ids: IDS[..count].iter().map(|id| ChangeId::parse(id).unwrap()).collect(),
        });
        let after = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let back_out = records(&b, "land-back-out");
        assert_pair(&back_out, "land-back-out", INTEGRATION, &b.landed[count - 1], &b.origin, IDS[count - 1]);
        assert!(back_out.iter().all(|record| (before..=after).contains(&record.at)));
        assert_eq!(b.f.git.read_ref(INTEGRATION).unwrap(), Some(b.origin.clone())); assert!(!b.path.exists());
    }
}

#[test]
fn rit_publish_logs_each_member_cleanup_in_order_and_the_newest_id_on_push_and_origin() {
    let b = Batch::new("second", "second");
    let remote_path = b.f.git.repo.join("batch-remote.git");
    raw(&b.f, &["init", "--bare", "--quiet", remote_path.to_str().unwrap()]);
    raw(&b.f, &["remote", "add", "origin", remote_path.to_str().unwrap()]);
    raw(&b.f, &["push", "--quiet", "origin", &format!("{}:{MAIN}", b.origin)]);
    for (i, name) in ["refs/mortar/squash/core", "refs/mortar/squash/second"].iter().enumerate() {
        raw(&b.f, &["update-ref", name, b.squashes[i].commit.as_str()]); b.land(i);
    }
    assert_eq!(locked(&b.f, || land::publish(&b.f.git, &refs(), ACTOR)), PublishAnswer::Published { origin: b.landed[1].clone() });
    let cleanups = records(&b, "publish-squash"); assert_eq!(cleanups.len(), 4);
    let zero = Oid::parse("0000000000000000000000000000000000000000").unwrap();
    for (i, name) in ["refs/mortar/squash/core", "refs/mortar/squash/second"].iter().enumerate() {
        assert_pair(&cleanups[i * 2..i * 2 + 2], "publish-squash", name, &b.squashes[i].commit, &zero, IDS[i]);
        assert_eq!(b.f.git.read_ref(name).unwrap(), None);
    }
    assert_pair(&records(&b, "publish-push"), "publish-push", MAIN, &b.origin, &b.landed[1], IDS[1]);
    assert_pair(&records(&b, "publish-origin"), "publish-origin", MAIN, &b.origin, &b.landed[1], IDS[1]);
    let marked = oplog::read_marked_log(&b.f.common).unwrap().records.into_iter()
        .filter(|m| m.record.kind == "publish-push").collect::<Vec<_>>();
    assert_eq!(marked.len(), 2); assert!(marked.iter().all(|m| m.remote.as_deref() == Some("origin")));
    let remote = gitio::Git { repo: remote_path, ..b.f.git.clone() };
    assert_eq!(remote.read_ref(MAIN).unwrap(), Some(b.landed[1].clone()));
    assert_eq!(b.f.git.read_ref(MAIN).unwrap(), Some(b.landed[1].clone())); assert!(!b.path.exists());
}

#[test]
fn rit_commit_files_logs_the_named_git_commit_and_newest_batch_identity() {
    let b = Batch::new("second", "second"); b.land(0); b.land(1);
    raw(&b.f, &["checkout", "--quiet", "integration"]);
    fs::write(b.f.git.repo.join("file"), b"named content\n").unwrap();
    let blob = b.f.git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
        stdin: Some(b"named content\n".to_vec()), ..RunOpts::default()
    }).unwrap();
    let blob = std::str::from_utf8(&blob.stdout).unwrap().trim();
    let tree = b.f.git.run(&["mktree"], &RunOpts {
        stdin: Some(format!("100644 blob {blob}\tfile\n").into_bytes()), ..RunOpts::default()
    }).unwrap();
    let tree = Oid::parse(std::str::from_utf8(&tree.stdout).unwrap().trim()).unwrap();
    let expected = b.f.git.commit_tree(&tree, std::slice::from_ref(&b.landed[1]), "batch named files\n", None, None).unwrap();
    assert_eq!(locked(&b.f, || land::commit_files(&b.f.git, &refs(), &[PathBuf::from("file")], "batch named files\n", ACTOR)),
        CommitFilesAnswer::Committed { commit: expected.clone() });
    assert_pair(&records(&b, "land-commit-files"), "land-commit-files", INTEGRATION, &b.landed[1], &expected, IDS[1]);
    assert_eq!(b.f.git.read_ref(INTEGRATION).unwrap(), Some(expected.clone()));
    assert_eq!(b.f.git.commit_info(&expected).unwrap().parents, vec![b.landed[1].clone()]);
    assert_eq!(land::read_landing_status(&b.f.git, &refs()).unwrap().unwrap().integration, expected);
    assert_eq!(land::read_landing_batch(&b.f.git).unwrap(), Some(b.members(2)));
    assert!(!fs::read_to_string(&b.path).unwrap().contains("\"pending\""));
}
