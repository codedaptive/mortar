//! WO-orc-join-it-1: SPEC-join WO-join-1, S-28/S-39; public join::replay with real git on both execution paths.
use gitio::{Git, Oid, Person, RunOpts};
use join::{replay, JoinError, ReplayResult};
use std::{fs, path::PathBuf};
use testkit::{write_git_shim, TestRepo};

mod overlap;

#[test]
fn clean_replay_keeps_each_author_message_and_trailer_and_moves_no_ref_on_either_path() {
    // Joins join::replay -> native replay or merge_tree/commit_info/commit_tree -> real commit chain.
    for fallback in [false, true] {
        let f = Fixture::new();
        let base_tree = f.tree(&[("file", b"base\n")]);
        let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
        let first_message = "first change\n\nbody\n\nMortar-Change-Id: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nMortar-Request: request-1\nMortar-Zone: topic\n";
        let second_message = "second change\n\nsecond body\n\nMortar-Request: request-1\nMortar-Zone: topic\nCustom-Trailer: preserve me\n";
        let first_author = author("Zoë Ångström", "1700000100 -0530");
        let second_author = author("Second Author", "1700000200 +0930");
        let first_tree = f.tree(&[("file", b"topic\n")]);
        let first = f.commit(&first_tree, &[base.clone()], first_message, &first_author);
        let second_tree = f.tree(&[("file", b"topic\n"), ("second", b"second payload\n")]);
        let tip = f.commit(&second_tree, &[first], second_message, &second_author);
        let onto_tree = f.tree(&[("file", b"base\n"), ("upstream", b"upstream payload\n")]);
        let onto = f.commit(&onto_tree, &[base.clone()], "onto\n", &author("Upstream", "1700000300 +0000"));
        f.anchor(&onto, &tip);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let new_tip = match replay(&git, &base, "refs/heads/topic", &onto).unwrap() {
            ReplayResult::Clean(id) => id,
            other => panic!("expected clean replay, fallback={fallback}: {other:?}"),
        };
        let second_info = f.git.commit_info(&new_tip).unwrap();
        assert_eq!(second_info.message, second_message);
        assert_eq!(second_info.author, second_author);
        assert_eq!(second_info.parents.len(), 1);
        let first_info = f.git.commit_info(&second_info.parents[0]).unwrap();
        assert_eq!(first_info.message, first_message);
        assert_eq!(first_info.author, first_author);
        assert_eq!(first_info.parents, vec![onto]);
        assert_eq!(first_info.tree, f.tree(&[("file", b"topic\n"), ("upstream", b"upstream payload\n")]));
        assert_eq!(second_info.tree, f.tree(&[("file", b"topic\n"), ("second", b"second payload\n"), ("upstream", b"upstream payload\n")]));
        assert_ne!(new_tip, tip);
        f.assert_state(&before);
        assert_path(&trace, fallback, false);
    }
}

#[test]
fn conflict_names_the_original_commit_replayed_tip_remaining_commits_tree_and_files_on_both_paths() {
    // Joins native conflict -> join fallback detail, or forced fallback -> exact public conflict result.
    for fallback in [false, true] {
        let f = Fixture::new();
        let base_tree = f.tree(&[("log.txt", b"base\n")]);
        let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
        let first_tree = f.tree(&[("log.txt", b"base\n"), ("clean", b"first replayed\n")]);
        let first = f.commit(&first_tree, &[base.clone()], "clean first\n", &author("First", "1700000100 +0000"));
        let bad_tree = f.tree(&[("log.txt", b"topic\n"), ("clean", b"first replayed\n")]);
        let bad = f.commit(&bad_tree, &[first], "conflict second\n", &author("Second", "1700000200 +0000"));
        let last_tree = f.tree(&[("log.txt", b"topic\n"), ("clean", b"first replayed\n"), ("later", b"must wait\n")]);
        let last = f.commit(&last_tree, &[bad.clone()], "remaining third\n", &author("Third", "1700000300 +0000"));
        let onto_tree = f.tree(&[("log.txt", b"onto\n")]);
        let onto = f.commit(&onto_tree, &[base.clone()], "onto\n", &author("Upstream", "1700000400 +0000"));
        f.anchor(&onto, &last);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        match replay(&git, &base, "refs/heads/topic", &onto).unwrap() {
            ReplayResult::Conflicted { commit, replayed_tip, remaining, tree, files } => {
                assert_eq!(commit, bad);
                assert_eq!(remaining, vec![last]);
                assert_eq!(files, vec![PathBuf::from("log.txt")]);
                let replayed = f.git.commit_info(&replayed_tip).unwrap();
                assert_eq!(replayed.parents, vec![onto]);
                assert_eq!(replayed.message, "clean first\n");
                assert_eq!(replayed.tree, f.tree(&[("log.txt", b"onto\n"), ("clean", b"first replayed\n")]));
                let content = String::from_utf8(f.blob(&tree, "log.txt")).unwrap();
                assert!(content.contains("<<<<<<<") && content.contains("=======") && content.contains(">>>>>>>"), "{content}");
                assert!(content.contains("onto\n") && content.contains("topic\n"), "{content}");
                assert_eq!(f.blob(&tree, "clean"), b"first replayed\n");
                assert_eq!(f.names(&tree), b"clean\0log.txt\0");
            }
            other => panic!("expected named conflict, fallback={fallback}: {other:?}"),
        }
        f.assert_state(&before);
        assert_path(&trace, fallback, true);
    }
}

#[test]
fn union_driver_is_read_from_onto_not_the_conflicting_worktree_attributes_on_both_paths() {
    // Joins join's onto attribute source -> native replay/merge-tree union -> exact result, untouched worktree.
    for fallback in [false, true] {
        let f = Fixture::new();
        let base_tree = f.tree(&[("log.txt", b"base\n")]);
        let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
        let topic_tree = f.tree(&[("log.txt", b"topic\n")]);
        let topic = f.commit(&topic_tree, &[base.clone()], "topic\n\nMortar-Request: request-1\n", &author("Topic", "1700000100 -0530"));
        let onto_tree = f.tree(&[("log.txt", b"onto\n"), (".gitattributes", b"log.txt merge=union\n")]);
        let onto = f.commit(&onto_tree, &[base.clone()], "onto union\n", &author("Upstream", "1700000200 +0000"));
        f.anchor(&onto, &topic);
        fs::write(f.repo.path.join(".gitattributes"), b"log.txt -merge\n").unwrap();
        assert_eq!(f.run(&["check-attr", "merge", "--", "log.txt"]), b"log.txt: merge: unset\n");
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let new = match replay(&git, &base, "refs/heads/topic", &onto).unwrap() {
            ReplayResult::Clean(id) => id,
            other => panic!("onto union must win over working-tree -merge: {other:?}"),
        };
        let info = f.git.commit_info(&new).unwrap();
        assert_eq!(info.parents, vec![onto]);
        assert_eq!(info.tree, f.tree(&[("log.txt", b"onto\ntopic\n"), (".gitattributes", b"log.txt merge=union\n")]));
        assert_eq!(f.blob(&new, "log.txt"), b"onto\ntopic\n");
        assert_eq!(info.author, author("Topic", "1700000100 -0530"));
        assert_eq!(info.message, "topic\n\nMortar-Request: request-1\n");
        assert_eq!(fs::read(f.repo.path.join(".gitattributes")).unwrap(), b"log.txt -merge\n");
        f.assert_state(&before);
        assert_path(&trace, fallback, false);
    }
}

#[test]
fn missing_onto_is_typed_refusal_before_any_merge_replay_or_commit_on_both_paths() {
    // Joins join's early onto check -> real object lookup -> no attempted replay/merge/commit and no ref movement.
    for fallback in [false, true] {
        let f = Fixture::new();
        let contents = f.tree(&[("file", b"base\n")]);
        let base = f.commit(&contents, &[], "base\n", &author("Base", "1700000000 +0000"));
        let tip = f.commit(&contents, &[base.clone()], "topic\n", &author("Topic", "1700000100 +0000"));
        f.anchor(&base, &tip);
        let missing = Oid::parse(&"0".repeat(40)).unwrap();
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        assert_eq!(replay(&git, &base, "refs/heads/topic", &missing).unwrap_err(), JoinError::OntoMissing(missing));
        assert!(fs::read_to_string(trace).unwrap_or_default().is_empty());
        f.assert_state(&before);
    }
}

struct Fixture { repo: TestRepo, git: Git }

impl Fixture {
    fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env.extend([
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_COMMITTER_NAME".into(), "Replay Committer".into()),
            ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
            ("GIT_COMMITTER_DATE".into(), "1700000999 +0000".into()),
        ]);
        Self { repo, git }
    }
    fn run(&self, args: &[&str]) -> Vec<u8> { self.git.run(args, &RunOpts::default()).unwrap().stdout }
    fn tree(&self, files: &[(&str, &[u8])]) -> Oid {
        let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), self.repo.path.join(".git/oracle-index").into_os_string())], ..RunOpts::default() };
        self.git.run(&["read-tree", "--empty"], &opts).unwrap();
        for (path, bytes) in files {
            let out = self.git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() }).unwrap();
            let id = std::str::from_utf8(&out.stdout).unwrap().trim_end();
            self.git.run(&["update-index", "--add", "--cacheinfo", "100644", id, path], &opts).unwrap();
        }
        oid(&self.git.run(&["write-tree"], &opts).unwrap().stdout)
    }
    fn commit(&self, tree: &Oid, parents: &[Oid], message: &str, person: &Person) -> Oid {
        let mut args = vec!["commit-tree".to_string(), tree.as_str().to_string()];
        for parent in parents { args.extend(["-p".into(), parent.as_str().into()]); }
        oid(&self.git.run(&args, &RunOpts {
            stdin: Some(message.as_bytes().to_vec()),
            env: vec![("GIT_AUTHOR_NAME".into(), person.name.clone().into()), ("GIT_AUTHOR_EMAIL".into(), person.email.clone().into()),
                ("GIT_AUTHOR_DATE".into(), person.date.clone().unwrap().into())], ..RunOpts::default()
        }).unwrap().stdout)
    }
    fn anchor(&self, onto: &Oid, topic: &Oid) {
        self.run(&["update-ref", "refs/heads/main", onto.as_str()]);
        self.run(&["update-ref", "refs/heads/topic", topic.as_str()]);
        self.run(&["checkout", "--force", "main"]);
        fs::write(self.repo.path.join("local-only"), b"local work must survive\n").unwrap();
    }
    fn state(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        (self.run(&["for-each-ref", "--format=%(refname) %(objectname)"]),
         fs::read(self.repo.path.join(".git/index")).unwrap(), self.run(&["rev-parse", "HEAD"]))
    }
    fn assert_state(&self, before: &(Vec<u8>, Vec<u8>, Vec<u8>)) {
        assert_eq!(&self.state(), before);
        assert_eq!(fs::read(self.repo.path.join("local-only")).unwrap(), b"local work must survive\n");
    }
    fn blob(&self, commit_or_tree: &Oid, path: &str) -> Vec<u8> { self.run(&["show", &format!("{commit_or_tree}:{path}")]) }
    fn names(&self, tree: &Oid) -> Vec<u8> { self.run(&["ls-tree", "-r", "--name-only", "-z", tree.as_str()]) }
    fn runner(&self, fallback: bool) -> Option<(Git, PathBuf)> {
        if !fallback && !self.git.replay_available().unwrap() {
            eprintln!("join native path unavailable; forced-fallback outcome remains exercised");
            return None;
        }
        eprintln!("join replay integration path: {}", if fallback { "forced fallback" } else { "native available" });
        let shim = self.repo.path.join(".git/replay-path.sh");
        let trace = self.repo.path.join(".git/replay-calls");
        // Only the capability probe is overridden; every merge, commit and native replay executes real git.
        write_git_shim(&shim, b"#!/bin/sh\nset -eu\nif [ \"${3-}\" = replay ] && [ \"${4-}\" = -h ]; then\n  if [ \"$MORTAR_FORCE_FALLBACK\" = 1 ]; then\n    printf 'forced unavailable replay\\n' >&2\n    exit 129\n  fi\nelse\n  for arg in \"$@\"; do\n    case \"$arg\" in replay|merge-tree|commit-tree) printf '%s\\n' \"$arg\" >>\"$MORTAR_REPLAY_TRACE\";; esac\n  done\nfi\nexec git \"$@\"\n").unwrap();
        let mut git = self.git.clone();
        git.program = shim;
        git.env.extend([("MORTAR_FORCE_FALLBACK".into(), if fallback { "1" } else { "0" }.into()),
            ("MORTAR_REPLAY_TRACE".into(), trace.clone().into_os_string())]);
        Some((git, trace))
    }
}

fn oid(bytes: &[u8]) -> Oid { Oid::parse(std::str::from_utf8(bytes).unwrap().trim_end()).unwrap() }
fn author(name: &str, date: &str) -> Person { Person { name: name.into(), email: "author@example.invalid".into(), date: Some(date.into()) } }
fn assert_path(trace: &PathBuf, fallback: bool, conflict: bool) {
    let calls = fs::read_to_string(trace).unwrap();
    assert_eq!(calls.lines().any(|line| line == "replay"), !fallback, "{calls}");
    assert_eq!(calls.lines().any(|line| line == "merge-tree"), fallback || conflict, "{calls}");
    if fallback || conflict { assert!(calls.lines().any(|line| line == "commit-tree"), "{calls}"); }
}

// WO-orc-join-it-2, SPEC-join WO-join-2 (S-20/S-21), plus S-43's public fallback refusal.
const JOIN_ZONE: &str = "it-zone";
const ZONE_REF: &str = "refs/heads/zone/it-zone";
const REQUEST_REF: &str = "refs/heads/request/task";
// WO-orc-join-it-3: the caller supplies attribution; lock/recovery cases remain deferred to cli.
const JOIN_ACTOR: &str = "join-it-3-caller";

#[test]
fn join_fast_forwards_only_the_zone_and_repeat_is_joined_without_another_op() {
    // Joins join -> protected oplog apply -> exact refs/log and inert repeat; caller-lock coverage belongs to cli.
    let f = Fixture::new();
    let initial = f.tree(&[("file", b"base\n")]);
    let base = f.commit(&initial, &[], "base\n", &author("Base", "1700000000 +0000"));
    let changed = f.tree(&[("file", b"request\n")]);
    let tip = f.commit(&changed, &[base.clone()], "request\n", &author("Topic", "1700000100 -0530"));
    join_refs(&f, &base, &tip);
    let before = f.state();
    assert_eq!(join::join(&f.git, &f.repo.path, JOIN_ZONE, REQUEST_REF, JOIN_ACTOR).unwrap(),
        join::JoinAnswer::FastForwarded { zone_ref: ZONE_REF.into(), from: base.clone(), to: tip.clone() });
    let expected = (moved_refs(&before.0, ZONE_REF, &base, &tip), before.1, before.2);
    f.assert_state(&expected);
    let common = f.git.common_dir().unwrap();
    let log = oplog::read_op_log(&common).unwrap();
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.records[0].phase, oplog::OpPhase::Intent);
    assert_eq!(log.records[1].phase, oplog::OpPhase::Done);
    assert_eq!(log.records[1].kind, "join-fast-forward");
    assert_eq!(log.records[1].result, Some(oplog::OpResult::Moved));
    assert_eq!(log.records[1].moves, vec![oplog::RefMove { refname: ZONE_REF.into(), old: Some(base), new: tip.clone() }]);
    assert_eq!(log.records[1].actor, "join-it-3-caller");
    let bytes = fs::read(oplog::oplog_path(&common)).unwrap();
    assert_eq!(join::join(&f.git, &f.repo.path, JOIN_ZONE, REQUEST_REF, JOIN_ACTOR).unwrap(),
        join::JoinAnswer::Joined { zone_ref: ZONE_REF.into(), request_tip: tip });
    f.assert_state(&expected);
    assert_eq!(fs::read(oplog::oplog_path(&common)).unwrap(), bytes);
}

#[test]
fn join_rebases_a_moved_zone_and_stops_with_only_the_request_ref_moved() {
    // Joins merge-base -> real replay -> request CAS, deliberately not a zone fast-forward or worktree checkout.
    let f = Fixture::new();
    let base_tree = f.tree(&[("file", b"base\n")]);
    let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
    let topic_tree = f.tree(&[("file", b"request\n")]);
    let message = "request change\n\nMortar-Request: task\nMortar-Zone: it-zone\n";
    let topic = f.commit(&topic_tree, &[base.clone()], message, &author("Zoë", "1700000100 -0530"));
    let zone_tree = f.tree(&[("file", b"base\n"), ("upstream", b"zone change\n")]);
    let zone = f.commit(&zone_tree, &[base], "zone moved\n", &author("Zone", "1700000200 +0000"));
    join_refs(&f, &zone, &topic);
    let before = f.state();
    let new = match join::join(&f.git, &f.repo.path, JOIN_ZONE, REQUEST_REF, JOIN_ACTOR).unwrap() {
        join::JoinAnswer::Rebased { request_ref, from, to } => { assert_eq!(request_ref, REQUEST_REF); assert_eq!(from, topic); to }
        other => panic!("must stop after request rebase: {other:?}"),
    };
    assert_ne!(new, topic);
    let info = f.git.commit_info(&new).unwrap();
    assert_eq!(info.parents, vec![zone.clone()]);
    assert_eq!(info.tree, f.tree(&[("file", b"request\n"), ("upstream", b"zone change\n")]));
    assert_eq!(info.message, message);
    assert_eq!(info.author, author("Zoë", "1700000100 -0530"));
    assert_eq!(f.git.read_ref(ZONE_REF).unwrap(), Some(zone));
    assert_eq!(f.state().0, moved_refs(&before.0, REQUEST_REF, &topic, &new));
    assert_eq!(fs::read(f.repo.path.join(".git/index")).unwrap(), before.1);
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(new));
    assert_eq!(fs::read(f.repo.path.join("file")).unwrap(), b"request\n");
    assert!(!f.repo.path.join("upstream").exists(), "the caller, not join, checks out the replayed request");
    let log = oplog::read_op_log(&f.git.common_dir().unwrap()).unwrap();
    assert_eq!(log.records.len(), 2);
    assert!(log.records.iter().all(|record| record.kind == "join-rebase"));
    assert_eq!(log.records[1].phase, oplog::OpPhase::Done);
    assert_eq!(log.records[1].moves[0].refname, REQUEST_REF);
    assert_eq!(log.records[1].actor, "join-it-3-caller");
}

#[test]
fn join_conflict_lists_files_writes_exact_resolve_state_and_moves_no_ref() {
    // Joins replay conflict -> mortar-join.json serialization/filesystem -> exact paused state and unchanged refs.
    let f = Fixture::new();
    let base_tree = f.tree(&[("conflict file.txt", b"base\n")]);
    let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
    let bad_tree = f.tree(&[("conflict file.txt", b"request\n")]);
    let bad = f.commit(&bad_tree, &[base.clone()], "conflicting change\n", &author("Topic", "1700000100 +0000"));
    let later_tree = f.tree(&[("conflict file.txt", b"request\n"), ("later", b"not replayed yet\n")]);
    let tip = f.commit(&later_tree, &[bad.clone()], "remaining\n", &author("Later", "1700000200 +0000"));
    let zone_tree = f.tree(&[("conflict file.txt", b"zone\n")]);
    let zone = f.commit(&zone_tree, &[base.clone()], "zone moved\n", &author("Zone", "1700000300 +0000"));
    join_refs(&f, &zone, &tip);
    let conflict_tree = match f.git.merge_tree(&zone, Some(&base), &zone, &bad).unwrap() {
        gitio::MergeTreeOutcome::Conflicted { tree, files } => { assert_eq!(files, vec![PathBuf::from("conflict file.txt")]); tree }
        other => panic!("fixture must be a real shared-line conflict: {other:?}"),
    };
    let before = f.state();
    let common = f.git.common_dir().unwrap();
    let state = match join::join(&f.git, &f.repo.path, JOIN_ZONE, REQUEST_REF, JOIN_ACTOR).unwrap() {
        join::JoinAnswer::Conflicted { commit, files, state } => {
            assert_eq!(commit, bad); assert_eq!(files, vec![PathBuf::from("conflict file.txt")]); state
        }
        other => panic!("expected paused join: {other:?}"),
    };
    assert_eq!(state, common.join("mortar-join.json"));
    let expected = format!("{{\"request_ref\":\"{REQUEST_REF}\",\"request_tip\":\"{tip}\",\"onto\":\"{zone}\",\"replayed_tip\":\"{zone}\",\"commit\":\"{bad}\",\"tree\":\"{conflict_tree}\",\"remaining\":[\"{tip}\"],\"files\":[\"conflict file.txt\"],\"files_written\":false,\"rebased_to\":null}}\n");
    assert_eq!(fs::read_to_string(state).unwrap(), expected);
    let conflicted = String::from_utf8(f.blob(&conflict_tree, "conflict file.txt")).unwrap();
    assert!(conflicted.contains("<<<<<<<") && conflicted.contains("zone\n") && conflicted.contains("request\n"), "{conflicted}");
    f.assert_state(&before);
    assert!(oplog::read_op_log(&common).unwrap().records.is_empty());
}

#[test]
fn join_missing_zone_or_request_is_named_without_history() {
    // Joins actual missing-ref observation -> named typed refusal without ref/index/file/history mutation.
    for missing_zone in [true, false] {
        let f = Fixture::new();
        let contents = f.tree(&[("file", b"base\n")]);
        let base = f.commit(&contents, &[], "base\n", &author("Base", "1700000000 +0000"));
        let topic = f.commit(&contents, &[base.clone()], "request\n", &author("Topic", "1700000100 +0000"));
        join_refs(&f, &base, &topic);
        f.run(&["checkout", "--force", "main"]);
        f.run(&["update-ref", "-d", if missing_zone { ZONE_REF } else { REQUEST_REF }]);
        let before = f.state();
        match (missing_zone, join::join(&f.git, &f.repo.path, JOIN_ZONE, REQUEST_REF, JOIN_ACTOR).unwrap_err()) {
            (true, join::JoinRunError::ZoneMissing(name)) => assert_eq!(name, ZONE_REF),
            (false, join::JoinRunError::RequestMissing(name)) => assert_eq!(name, REQUEST_REF),
            (_, other) => panic!("wrong missing-ref refusal: {other:?}"),
        }
        f.assert_state(&before);
        let common = f.git.common_dir().unwrap();
        assert!(oplog::read_op_log(&common).unwrap().records.is_empty());
    }
}

// S-47 / Orc #843/#848: caller-held lock and kill-mid-join/recover integration belongs to cli,
// once cli exists. No join-owned-lock crash fixture is kept here.

#[test]
fn fallback_refuses_empty_author_date_naming_the_original_commit_before_merge_or_commit_tree() {
    // S-43: actual malformed commit remains readable by gitio, but public join::replay must not silently re-date it.
    let f = Fixture::new();
    let contents = f.tree(&[("file", b"base\n")]);
    let base = f.commit(&contents, &[], "base\n", &author("Base", "1700000000 +0000"));
    let raw = format!("tree {contents}\nparent {base}\nauthor No Date <no-date@example.invalid> \ncommitter Committer <committer@example.invalid> 1700000200 +0000\n\nrequest with empty date\n");
    let out = f.git.run(&["hash-object", "--literally", "-t", "commit", "-w", "--stdin"], &RunOpts {
        stdin: Some(raw.as_bytes().to_vec()), ..RunOpts::default()
    }).unwrap();
    let malformed = oid(&out.stdout);
    assert_eq!(f.git.commit_info(&malformed).unwrap().author.date.as_deref(), Some(""));
    f.anchor(&base, &malformed);
    let before = f.state();
    let commits = f.run(&["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]);
    let (git, trace) = f.runner(true).unwrap();
    assert_eq!(replay(&git, &base, "refs/heads/topic", &base).unwrap_err(), JoinError::EmptyAuthorDate(malformed));
    assert!(fs::read_to_string(trace).unwrap_or_default().is_empty(), "refusal must precede merge-tree and commit-tree");
    assert_eq!(f.run(&["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]), commits);
    f.assert_state(&before);
}

fn join_refs(f: &Fixture, zone: &Oid, topic: &Oid) {
    f.run(&["update-ref", ZONE_REF, zone.as_str()]);
    f.run(&["update-ref", REQUEST_REF, topic.as_str()]);
    f.anchor(topic, topic);
    f.run(&["checkout", "--force", "request/task"]);
}

fn moved_refs(before: &[u8], name: &str, old: &Oid, new: &Oid) -> Vec<u8> {
    let old_line = format!("{name} {old}\n");
    let before = std::str::from_utf8(before).unwrap();
    assert!(before.contains(&old_line));
    before.replace(&old_line, &format!("{name} {new}\n")).into_bytes()
}
