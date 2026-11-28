//! Save joined to real snapshot/gitio/oplog: the change ID read back from HEAD, and index repair where none is needed.
use crate::support::{blob, hook_script, run, tree_names, write, Fixture};
use gitio::{HookRefusal, Oid, RunOpts};
use oplog::{oplog_path, read_mortar_trailers, read_op_log, ChangeIdError, MortarTrailerError, OpPhase, OpResult, RefMove};
use snapshot::{save, save_with_hooks, SaveHooksError, SaveOutcome, SnapshotError, DEFAULT_SIZE_CAP};
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
    // This index already matches HEAD; saves keep that tree and entries, not necessarily stat/cache bytes.
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
    // Joins actual HEAD trailers -> missing-ID refusal -> snapshot history without branch/index mutation.
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
    // No retry/repair assertion here: the retry is in save_repair.rs.
}

#[test]
fn hooks_refusing_pre_commit_stops_save_with_its_output_and_leaves_branch_index_and_commits_but_keeps_its_snapshot() {
    // Case 1. The folder differs from HEAD and has no snapshot yet, so save's snapshot step writes one before the
    // hook refuses; that snapshot stays and nothing of the save does.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"refused by pre-commit\n");
    hook_script(&f, "pre-commit", b"printf 'out-text\\n'\nprintf 'err-text\\n' >&2\nexit 3\n");
    let before = HookState::read(&f);
    let refusal = hook_refusal(save_with_hooks(&f.git, &f.repo.path, &f.target, "Refused save", &[], DEFAULT_SIZE_CAP));
    assert_eq!(refusal, HookRefusal { hook: "pre-commit", code: Some(3), stdout: b"out-text\n".to_vec(), stderr: b"err-text\n".to_vec() });
    assert_refusal_kept_only_its_snapshot(&f, before, b"refused by pre-commit\n");
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"refused by pre-commit\n");
}

#[test]
fn hooks_refusing_commit_msg_stops_save_and_leaves_branch_index_and_commits_but_keeps_its_snapshot() {
    // Case 3, on a changed folder as in case 1.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"refused by commit-msg\n");
    hook_script(&f, "commit-msg", b"exit 1\n");
    let before = HookState::read(&f);
    let refusal = hook_refusal(save_with_hooks(&f.git, &f.repo.path, &f.target, "Refused message", &[], DEFAULT_SIZE_CAP));
    assert_eq!((refusal.hook, refusal.code), ("commit-msg", Some(1)));
    assert_refusal_kept_only_its_snapshot(&f, before, b"refused by commit-msg\n");
}

#[test]
fn hooks_commit_msg_rewrite_of_the_subject_is_the_saved_commits_message() {
    // Case 4 (save): the hook replaces the first line and keeps the rest, the Mortar trailers included.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"rewritten subject\n");
    hook_script(&f, "commit-msg", b"{ printf 'hooked subject\\n'; tail -n +2 \"$1\"; } > \"$1.new\"\nmv \"$1.new\" \"$1\"\n");
    let saved = save_with_hooks(&f.git, &f.repo.path, &f.target, "Built subject", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(saved.written);
    let id = saved.change_id.unwrap();
    let message = f.git.commit_info(&saved.commit).unwrap().message;
    assert_eq!(message, format!("hooked subject\n\nMortar-Change-Id: {}\nMortar-Request: request-17\nMortar-Zone: snapshot-zone\n", id.as_str()));
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(saved.commit));
}

#[test]
fn hooks_commit_msg_that_drops_the_trailers_still_saves_each_once_and_the_next_save_reuses_the_id() {
    // Case 6: the hook keeps only the first line; save adds the Mortar trailers back.
    let f = Fixture::new(&[]);
    hook_script(&f, "commit-msg", b"head -n 1 \"$1\" > \"$1.new\"\nmv \"$1.new\" \"$1\"\n");
    write(&f.repo.path, "root.txt", b"first trailer save\n");
    let first = save_with_hooks(&f.git, &f.repo.path, &f.target, "First trailer save", &[], DEFAULT_SIZE_CAP).unwrap();
    let id = first.change_id.clone().unwrap();
    let message = f.git.commit_info(&first.commit).unwrap().message;
    assert!(message.starts_with("First trailer save\n"), "{message}");
    assert_eq!(trailer_lines(&message, "Mortar-Change-Id"), [format!("Mortar-Change-Id: {}", id.as_str())]);
    assert_eq!(trailer_lines(&message, "Mortar-Request"), ["Mortar-Request: request-17"]);
    assert_eq!(trailer_lines(&message, "Mortar-Zone"), ["Mortar-Zone: snapshot-zone"]);
    write(&f.repo.path, "root.txt", b"second trailer save\n");
    let second = save_with_hooks(&f.git, &f.repo.path, &f.target, "Second trailer save", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(second.written);
    assert_eq!(second.change_id, Some(id.clone()));
    assert_eq!(f.git.commit_info(&second.commit).unwrap().parents, vec![first.commit]);
    let message = f.git.commit_info(&second.commit).unwrap().message;
    assert_eq!(trailer_lines(&message, "Mortar-Change-Id"), [format!("Mortar-Change-Id: {}", id.as_str())]);
}

#[test]
fn hooks_hookless_save_writes_its_commit_with_a_refusing_pre_commit_installed() {
    // Case 8: save runs no hooks.
    let f = Fixture::new(&[]);
    hook_script(&f, "pre-commit", b"exit 1\n");
    write(&f.repo.path, "root.txt", b"saved past the hook\n");
    let saved = save(&f.git, &f.repo.path, &f.target, "Hookless save", &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(saved.written);
    assert_eq!(f.git.read_ref("refs/heads/main").unwrap(), Some(saved.commit.clone()));
    assert_eq!(f.git.commit_info(&saved.commit).unwrap().parents, vec![f.head.clone()]);
    assert_eq!(blob(&f.git, &saved.tree, "root.txt"), b"saved past the hook\n");
}

/// What a refused save must leave as it was: HEAD, the branch, the index bytes and the op log's record count; and
/// the commit objects, compared by choice as commits only, since the snapshot step's new trees and blobs may stay.
struct HookState { head: Option<Oid>, branch: Option<Oid>, index: Vec<u8>, records: usize, commits: Vec<String> }

impl HookState {
    fn read(f: &Fixture) -> Self {
        Self {
            head: f.git.rev_parse("HEAD").unwrap(),
            branch: f.git.read_ref("refs/heads/main").unwrap(),
            index: fs::read(f.common.join("index")).unwrap(),
            records: if oplog_path(&f.common).exists() { read_op_log(&f.common).unwrap().records.len() } else { 0 },
            commits: commit_ids(f),
        }
    }
}

/// A hook refusal after save's snapshot step wrote: HEAD, the branch and the index as `before`; the snapshot ref at a
/// new commit holding `work`, moved by one snapshot op (its intent and done records); and no commit but that one.
fn assert_refusal_kept_only_its_snapshot(f: &Fixture, before: HookState, work: &[u8]) {
    let after = HookState::read(f);
    assert_eq!((&after.head, &after.branch, &after.index), (&before.head, &before.branch, &before.index));
    let snap = f.git.read_ref(&f.refname).unwrap().expect("the snapshot step moved the snapshot ref");
    assert_eq!(blob(&f.git, &f.git.commit_info(&snap).unwrap().tree, "root.txt"), work);
    let mut commits = before.commits;
    commits.push(snap.as_str().to_owned());
    commits.sort();
    assert_eq!(after.commits, commits, "the snapshot commit is the only new commit");
    let records = read_op_log(&f.common).unwrap().records;
    assert_eq!(records.len(), before.records + 2);
    let op = &records[before.records..];
    assert_eq!((op[0].phase, op[1].phase), (OpPhase::Intent, OpPhase::Done));
    assert_eq!(op[0].op_id, op[1].op_id);
    for record in op {
        assert_eq!(record.kind, "snapshot");
        assert_eq!(record.moves, vec![RefMove { refname: f.refname.clone(), old: None, new: snap.clone() }]);
    }
}

fn hook_refusal(result: Result<SaveOutcome, SaveHooksError>) -> HookRefusal {
    match result {
        Err(SaveHooksError::HookRefused(refusal)) => refusal,
        other => panic!("expected HookRefused, got {other:?}"),
    }
}

fn trailer_lines<'a>(message: &'a str, key: &str) -> Vec<&'a str> {
    message.lines().filter(|line| line.strip_prefix(key).is_some_and(|rest| rest.starts_with(": "))).collect()
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
