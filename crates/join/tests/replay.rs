//! WO-orc-join-it-1: SPEC-join WO-join-1, S-28/S-39; public join::replay with real git on both execution paths.
use gitio::{Git, Oid, Person, RunOpts};
use join::{replay, JoinError, ReplayResult};
use std::{fs, path::PathBuf};
use testkit::{write_git_shim, TestRepo};

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
