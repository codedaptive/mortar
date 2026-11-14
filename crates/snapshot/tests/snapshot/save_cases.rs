//! WO-orc-snapshot-it-6: save joined to real snapshot/gitio/oplog, S-34 and the non-repair part of S-41.
use crate::support::{blob, run, tree_names, write, Fixture};
use gitio::{Oid, RunOpts};
use oplog::{oplog_path, read_mortar_trailers, read_op_log, ChangeIdError, MortarTrailerError, OpPhase, OpResult};
use snapshot::{save, SnapshotError, DEFAULT_SIZE_CAP};
use std::fs;

#[test]
fn save_on_zone_tip_draws_a_new_id_leaves_status_clean_and_next_save_reuses_the_requests_id() {
    // Joins save -> snapshot tree -> trailer/new ID selection -> branch apply -> real-index set and repeat save.
    let f = Fixture::new(&[]);
    let zone_id = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";
    let zone_head = seed_head(&f, &format!("zone tip\n\nMortar-Change-Id: {zone_id}\nMortar-Request: zone-root\nMortar-Zone: snapshot-zone\n"));
    write(&f.repo.path, "root.txt", b"first saved change\n");
    write(&f.repo.path, "added.txt", b"new saved file\n");
    let caller_index = f.common.join("caller-index");
    let caller_bytes = fs::read(f.common.join("index")).unwrap();
    fs::write(&caller_index, &caller_bytes).unwrap();
    let mut caller = f.git.clone();
    caller.env.push(("GIT_INDEX_FILE".into(), caller_index.clone().into_os_string()));
    let first = save(&caller, &f.repo.path, &f.target, "Save the first change", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(first.written);
    assert!(first.refused.is_empty());
    let id = first.change_id.clone().unwrap();
    assert_ne!(id.as_str(), zone_id);
    assert_eq!(id.as_str().len(), 32);
    assert!(id.as_str().chars().all(|c| ('k'..='z').contains(&c)));
    let info = f.git.commit_info(&first.commit).unwrap();
    assert_eq!(info.parents, vec![zone_head]);
    assert!(info.message.starts_with("Save the first change\n\n"), "{}", info.message);
    let trailers = read_mortar_trailers(&f.git, &info.message).unwrap();
    assert_eq!(trailers.change_id, Some(id.clone()));
    assert_eq!(trailers.request.as_deref(), Some(f.target.request.as_str()));
    assert_eq!(trailers.zone.as_deref(), Some(f.target.zone.as_str()));
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(first.commit.clone()));
    assert_eq!(tree_names(&f.git, &first.tree), ["added.txt", "root.txt"]);
    assert_eq!(blob(&f.git, &first.tree, "root.txt"), b"first saved change\n");
    assert_eq!(blob(&f.git, &first.tree, "added.txt"), b"new saved file\n");
    assert_eq!(run(&f.git, &["write-tree"]), format!("{}\n", first.tree).as_bytes());
    assert!(run(&f.git, &["status", "--porcelain=v1", "-z"]).is_empty());
    assert_eq!(fs::read(&caller_index).unwrap(), caller_bytes, "save must set the folder's own index, not caller GIT_INDEX_FILE");
    let first_log = read_op_log(&f.common).unwrap();
    assert_eq!(first_log.records.len(), 4);
    assert_eq!(first_log.records[2].kind, "save");
    assert_eq!(first_log.records[3].phase, OpPhase::Done);
    assert_eq!(first_log.records[3].result, Some(OpResult::Moved));
    assert_eq!(first_log.records[3].actor, f.target.actor);
    assert_eq!(first_log.records[3].moves[0].new, first.commit);
    write(&f.repo.path, "root.txt", b"second saved change\n");
    let second = save(&f.git, &f.repo.path, &f.target, "Save the second change", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(second.written);
    assert_eq!(second.change_id, Some(id.clone()));
    let info = f.git.commit_info(&second.commit).unwrap();
    assert_eq!(info.parents, vec![first.commit]);
    assert_eq!(read_mortar_trailers(&f.git, &info.message).unwrap().change_id, Some(id));
    assert_eq!(blob(&f.git, &second.tree, "root.txt"), b"second saved change\n");
    assert!(run(&f.git, &["status", "--porcelain=v1", "-z"]).is_empty());
    assert_eq!(read_op_log(&f.common).unwrap().records.len(), 8);
}

#[test]
fn unchanged_save_keeps_index_at_heads_tree_and_entries_and_repeat_writes_no_history() {
    // S-41/#843: this index already matches HEAD; saves keep that tree and entries, not necessarily stat/cache bytes.
    // Repair of a deliberately stale index is proved separately in save_repair.rs.
    let f = Fixture::new(&[]);
    let head_tree = run(&f.git, &["rev-parse", "HEAD^{tree}"]);
    let head_entries = run(&f.git, &["ls-tree", "-r", "--format=%(objectmode) %(objectname) 0\t%(path)", "HEAD"]);
    let first = save(&f.git, &f.repo.path, &f.target, "Nothing new", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(!first.written);
    assert_eq!(first.commit, f.head);
    assert!(first.change_id.is_none());
    assert!(first.refused.is_empty());
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
    assert_eq!(run(&f.git, &["write-tree"]), head_tree);
    assert_eq!(run(&f.git, &["ls-files", "-s"]), head_entries);
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 2, "the initial snapshot may be initialized, but save must not move the branch");
    assert!(records.iter().all(|record| record.kind == "snapshot"));
    let log = fs::read(oplog_path(&f.common)).unwrap();
    let commits = commit_ids(&f);
    let again = save(&f.git, &f.repo.path, &f.target, "Still nothing", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(!again.written);
    assert_eq!(again.commit, f.head);
    assert!(again.change_id.is_none());
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), log);
    assert_eq!(commit_ids(&f), commits);
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
    assert_eq!(run(&f.git, &["write-tree"]), head_tree);
    assert_eq!(run(&f.git, &["ls-files", "-s"]), head_entries);
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"base\n");
}

#[test]
fn this_requests_head_without_change_id_refuses_after_snapshot_without_saving_a_branch_commit() {
    // Joins actual HEAD trailers -> S-34 missing ID refusal -> snapshot history without branch/index mutation.
    let f = Fixture::new(&[]);
    let head = seed_head(&f, "ours without id\n\nMortar-Request: request-17\nMortar-Zone: snapshot-zone\n");
    write(&f.repo.path, "root.txt", b"work preserved in snapshot\n");
    let before = commit_ids(&f);
    let index = fs::read(f.common.join("index")).unwrap();
    match save(&f.git, &f.repo.path, &f.target, "Must not invent an id", &[], DEFAULT_SIZE_CAP).unwrap_err() {
        SnapshotError::ChangeIdMissing { head: actual } => assert_eq!(actual, head),
        other => panic!("expected ChangeIdMissing, got {other:?}"),
    }
    assert_snapshot_only_error(&f, &head, &index, &before, b"work preserved in snapshot\n");
}

#[test]
fn malformed_change_id_in_real_head_trailers_is_typed_and_preserves_branch_index_and_work() {
    // Joins real git interpret-trailers -> oplog malformed-ID decision -> save Trailers, without a branch save.
    let f = Fixture::new(&[]);
    let head = seed_head(&f, "malformed id\n\nMortar-Change-Id: not-an-id\nMortar-Request: request-17\nMortar-Zone: snapshot-zone\n");
    write(&f.repo.path, "root.txt", b"malformed-head work stays\n");
    let before = commit_ids(&f);
    let index = fs::read(f.common.join("index")).unwrap();
    match save(&f.git, &f.repo.path, &f.target, "Must refuse malformed id", &[], DEFAULT_SIZE_CAP).unwrap_err() {
        SnapshotError::Trailers(MortarTrailerError::ChangeId(ChangeIdError::Malformed { text, reason })) => {
            assert_eq!(text, "not-an-id");
            assert!(!reason.is_empty());
        }
        other => panic!("expected real malformed trailer error, got {other:?}"),
    }
    assert_snapshot_only_error(&f, &head, &index, &before, b"malformed-head work stays\n");
}

#[test]
fn detached_and_unborn_heads_are_named_before_any_snapshot_commit_ref_or_op_is_written() {
    // Joins Git::head -> save's pre-snapshot refusal -> real object/ref/log/index preservation.
    for detached in [true, false] {
        let f = Fixture::new(&[]);
        if detached { run(&f.git, &["checkout", "--detach", f.head.as_str()]); }
        else { run(&f.git, &["update-ref", "-d", "refs/heads/main", f.head.as_str()]); }
        let before = commit_ids(&f);
        let index = fs::read(f.common.join("index")).unwrap();
        let refs = run(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]);
        let error = save(&f.git, &f.repo.path, &f.target, "No branch", &[], DEFAULT_SIZE_CAP).unwrap_err();
        match (detached, error) {
            (true, SnapshotError::DetachedHead { folder }) | (false, SnapshotError::UnbornHead { folder }) => assert_eq!(folder, f.repo.path),
            (_, other) => panic!("wrong HEAD refusal: {other:?}"),
        }
        assert_eq!(commit_ids(&f), before);
        assert_eq!(fs::read(f.common.join("index")).unwrap(), index);
        assert_eq!(run(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]), refs);
        assert!(!oplog_path(&f.common).exists());
        assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"base\n");
    }
}

#[test]
fn blank_and_multiline_sentences_refuse_before_snapshot_or_any_history_write() {
    // Joins save_message (through save) -> actual untouched HEAD/index/ref/log/object set and dirty files.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"must not be lost\n");
    let before = commit_ids(&f);
    let index = fs::read(f.common.join("index")).unwrap();
    for sentence in ["", " \t", "\n", "subject\nbody", "subject\rbody"] {
        match save(&f.git, &f.repo.path, &f.target, sentence, &[], DEFAULT_SIZE_CAP).unwrap_err() {
            SnapshotError::BadSentence { reason } => assert!(!reason.is_empty()),
            other => panic!("expected BadSentence for {sentence:?}, got {other:?}"),
        }
        assert_eq!(commit_ids(&f), before);
        assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
        assert_eq!(f.git.read_ref(&f.refname).unwrap(), None);
        assert_eq!(fs::read(f.common.join("index")).unwrap(), index);
        assert!(!oplog_path(&f.common).exists());
        assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"must not be lost\n");
    }
}

#[test]
fn real_index_lock_produces_index_not_set_after_logged_branch_move_without_touching_files_or_old_index() {
    // Joins save -> successful oplog apply -> real read-tree index-lock failure -> IndexNotSet commit identity.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"committed but index step blocked\n");
    let index = fs::read(f.common.join("index")).unwrap();
    let lock = f.common.join("index.lock");
    fs::write(&lock, b"owned integration lock\n").unwrap();
    let saved = match save(&f.git, &f.repo.path, &f.target, "Save before index failure", &[], DEFAULT_SIZE_CAP).unwrap_err() {
        SnapshotError::IndexNotSet { commit, error } => {
            assert!(error.code.is_some_and(|code| code != 0));
            assert!(error.stderr.contains("index.lock"), "{error:?}");
            commit
        }
        other => panic!("expected real index-lock failure, got {other:?}"),
    };
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(saved.clone()));
    let info = f.git.commit_info(&saved).unwrap();
    assert_eq!(info.parents, vec![f.head.clone()]);
    assert_eq!(blob(&f.git, &info.tree, "root.txt"), b"committed but index step blocked\n");
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index);
    assert_eq!(fs::read(&lock).unwrap(), b"owned integration lock\n");
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"committed but index step blocked\n");
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 4);
    assert_eq!(records[3].kind, "save");
    assert_eq!(records[3].phase, OpPhase::Done);
    assert_eq!(records[3].result, Some(OpResult::Moved));
    assert_eq!(records[3].moves[0].new, saved);
    fs::remove_file(lock).unwrap();
    // No retry/repair assertion here: that integration is deferred until WO-snapshot-3b (Orc #723).
}

fn seed_head(f: &Fixture, message: &str) -> Oid {
    let tree = String::from_utf8(run(&f.git, &["rev-parse", "HEAD^{tree}"])).unwrap();
    let out = f.git.run(&["commit-tree", tree.trim_end(), "-p", f.head.as_str()], &RunOpts {
        stdin: Some(message.as_bytes().to_vec()), ..RunOpts::default()
    }).unwrap();
    let head = Oid::parse(std::str::from_utf8(&out.stdout).unwrap().trim_end()).unwrap();
    run(&f.git, &["update-ref", "refs/heads/main", head.as_str(), f.head.as_str()]);
    head
}

fn commit_ids(f: &Fixture) -> Vec<String> {
    let bytes = run(&f.git, &["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]);
    let mut ids: Vec<_> = std::str::from_utf8(&bytes).unwrap().lines().filter_map(|line| line.strip_prefix("commit ").map(str::to_owned)).collect();
    ids.sort(); ids
}

fn assert_snapshot_only_error(f: &Fixture, head: &Oid, index: &[u8], before: &[String], work: &[u8]) {
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(head.clone()));
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index);
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), work);
    let snapshot_commit = f.git.read_ref(&f.refname).unwrap().unwrap();
    let info = f.git.commit_info(&snapshot_commit).unwrap();
    assert_eq!(blob(&f.git, &info.tree, "root.txt"), work);
    assert_eq!(commit_ids(f).len(), before.len() + 1, "only the documented snapshot commit may be created");
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| record.kind == "snapshot"));
}
