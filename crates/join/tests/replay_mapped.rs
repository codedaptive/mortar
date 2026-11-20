//! Public join::replay_mapped with real git, under both runner setups (native replay
//! available, and forced unavailable). replay_mapped always replays commit by commit, so neither setup may reach
//! git's replay; each source commit maps to the running tip after it. Its own Fixture, as join_rit keeps its own.
use gitio::{Git, Oid, Person, RunOpts};
use join::{replay, replay_mapped, MappedCommit, ReplayResult};
use std::{fs, path::PathBuf};
use testkit::{write_git_shim, TestRepo};

#[test]
fn clean_range_of_two_maps_each_commit_to_its_replayed_commit_and_the_last_to_the_tip() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let first_message = "first change\n\nMortar-Change-Id: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nMortar-Zone: topic\n";
        let first = f.commit(&f.tree(&[("file", b"topic\n")]), std::slice::from_ref(&base), first_message, &author("Zoë Ångström", "1700000100 -0530"));
        let second_tree = f.tree(&[("file", b"topic\n"), ("second", b"second payload\n")]);
        let second = f.commit(&second_tree, std::slice::from_ref(&first), "second change\n", &author("Second", "1700000200 +0930"));
        let onto_tree = f.tree(&[("file", b"base\n"), ("upstream", b"upstream payload\n")]);
        let onto = f.commit(&onto_tree, std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000300 +0000"));
        f.anchor(&onto, &second);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        let tip = clean(&out.result, fallback);
        let mapped = pairs(&out.mapped);
        assert_eq!(mapped.len(), 2, "fallback={fallback}");
        assert_eq!((mapped[0].0.clone(), mapped[1].0.clone()), (first.clone(), second.clone()), "oldest first");
        assert_eq!(mapped[1].1, tip, "the last entry is the Clean tip, fallback={fallback}");
        let one = f.git.commit_info(&mapped[0].1).unwrap();
        assert_eq!(one.parents, vec![onto.clone()]);
        assert_eq!(one.message, first_message);
        assert_eq!(one.author, author("Zoë Ångström", "1700000100 -0530"));
        assert_eq!(one.tree, f.tree(&[("file", b"topic\n"), ("upstream", b"upstream payload\n")]));
        let two = f.git.commit_info(&tip).unwrap();
        assert_eq!(two.parents, vec![mapped[0].1.clone()]);
        assert_eq!(two.message, "second change\n");
        assert_eq!(two.author, author("Second", "1700000200 +0930"));
        assert_eq!(two.tree, f.tree(&[("file", b"topic\n"), ("second", b"second payload\n"), ("upstream", b"upstream payload\n")]));
        assert_ne!(mapped[0].1, first);
        f.assert_state(&before);
        assert_commit_by_commit(&trace, true);
    }
}

#[test]
fn emptied_first_commit_maps_to_onto_and_the_next_commit_sits_on_onto() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let onto_tree = f.tree(&[("file", b"base\n"), ("u", b"shared\n")]);
        let onto = f.commit(&onto_tree, std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000400 +0000"));
        let emptied = f.commit(&onto_tree, std::slice::from_ref(&base), "emptied first\n", &author("First", "1700000100 +0000"));
        let real_tree = f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("t", b"topic\n")]);
        let real = f.commit(&real_tree, std::slice::from_ref(&emptied), "real second\n", &author("Second", "1700000200 +0000"));
        f.anchor(&onto, &real);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        let tip = clean(&out.result, fallback);
        assert_eq!(pairs(&out.mapped), vec![(emptied, onto.clone()), (real, tip.clone())], "fallback={fallback}");
        let info = f.git.commit_info(&tip).unwrap();
        assert_eq!(info.parents, vec![onto.clone()]);
        assert_eq!(info.message, "real second\n");
        assert_eq!(info.author, author("Second", "1700000200 +0000"));
        assert_eq!(info.tree, f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("t", b"topic\n")]));
        f.assert_state(&before);
        assert_commit_by_commit(&trace, true);
    }
}

#[test]
fn emptied_middle_commit_maps_to_the_previous_replayed_commit() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let onto = f.commit(&f.tree(&[("file", b"base\n"), ("u", b"shared\n")]), std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000400 +0000"));
        let one = f.commit(&f.tree(&[("file", b"base\n"), ("t", b"one\n")]), std::slice::from_ref(&base), "real first\n", &author("First", "1700000100 +0000"));
        let two = f.commit(&f.tree(&[("file", b"base\n"), ("t", b"one\n"), ("u", b"shared\n")]), std::slice::from_ref(&one), "emptied middle\n", &author("Second", "1700000200 +0000"));
        let three_tree = f.tree(&[("file", b"base\n"), ("t", b"one\n"), ("u", b"shared\n"), ("v", b"three\n")]);
        let three = f.commit(&three_tree, std::slice::from_ref(&two), "real third\n", &author("Third", "1700000300 +0000"));
        f.anchor(&onto, &three);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        let tip = clean(&out.result, fallback);
        let mapped = pairs(&out.mapped);
        assert_eq!(mapped.len(), 3, "fallback={fallback}");
        let first_replayed = mapped[0].1.clone();
        assert_eq!(mapped, vec![(one, first_replayed.clone()), (two, first_replayed.clone()), (three, tip.clone())], "fallback={fallback}");
        let first_info = f.git.commit_info(&first_replayed).unwrap();
        assert_eq!(first_info.parents, vec![onto.clone()]);
        assert_eq!(first_info.message, "real first\n");
        assert_eq!(first_info.tree, f.tree(&[("file", b"base\n"), ("t", b"one\n"), ("u", b"shared\n")]));
        let tip_info = f.git.commit_info(&tip).unwrap();
        assert_eq!(tip_info.parents, vec![first_replayed]);
        assert_eq!(tip_info.message, "real third\n");
        assert_eq!(tip_info.author, author("Third", "1700000300 +0000"));
        assert_eq!(tip_info.tree, f.tree(&[("file", b"base\n"), ("t", b"one\n"), ("u", b"shared\n"), ("v", b"three\n")]));
        f.assert_state(&before);
        assert_commit_by_commit(&trace, true);
    }
}

#[test]
fn emptied_last_commit_maps_to_the_previous_replayed_commit_which_is_the_tip() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let onto = f.commit(&f.tree(&[("file", b"base\n"), ("u", b"shared\n")]), std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000400 +0000"));
        let one = f.commit(&f.tree(&[("file", b"base\n"), ("t", b"one\n")]), std::slice::from_ref(&base), "real first\n", &author("First", "1700000100 +0000"));
        let two = f.commit(&f.tree(&[("file", b"base\n"), ("t", b"one\n"), ("u", b"shared\n")]), std::slice::from_ref(&one), "emptied last\n", &author("Second", "1700000200 +0000"));
        f.anchor(&onto, &two);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        let tip = clean(&out.result, fallback);
        assert_eq!(pairs(&out.mapped), vec![(one, tip.clone()), (two, tip.clone())], "fallback={fallback}");
        let info = f.git.commit_info(&tip).unwrap();
        assert_eq!(info.parents, vec![onto.clone()]);
        assert_eq!(info.message, "real first\n");
        assert_eq!(info.author, author("First", "1700000100 +0000"));
        assert_eq!(info.tree, f.tree(&[("file", b"base\n"), ("t", b"one\n"), ("u", b"shared\n")]));
        f.assert_state(&before);
        assert_commit_by_commit(&trace, true);
    }
}

#[test]
fn every_commit_dropped_is_clean_onto_and_every_entry_maps_to_onto() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let onto_tree = f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("v", b"also shared\n")]);
        let onto = f.commit(&onto_tree, std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000400 +0000"));
        let one = f.commit(&f.tree(&[("file", b"base\n"), ("u", b"shared\n")]), std::slice::from_ref(&base), "emptied one\n", &author("First", "1700000100 +0000"));
        let two = f.commit(&onto_tree, std::slice::from_ref(&one), "emptied two\n", &author("Second", "1700000200 +0000"));
        f.anchor(&onto, &two);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        assert_eq!(out.result, ReplayResult::Clean(onto.clone()), "fallback={fallback}");
        assert_eq!(pairs(&out.mapped), vec![(one, onto.clone()), (two, onto.clone())], "fallback={fallback}");
        f.assert_state(&before);
        assert_commit_by_commit(&trace, true);
    }
}

#[test]
fn already_empty_source_commit_is_kept_as_a_new_commit_with_its_parents_tree() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let onto = f.commit(&f.tree(&[("file", b"base\n"), ("upstream", b"up\n")]), std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000400 +0000"));
        let topic_tree = f.tree(&[("file", b"topic\n")]);
        let one = f.commit(&topic_tree, std::slice::from_ref(&base), "real first\n", &author("First", "1700000100 +0000"));
        let two = f.commit(&topic_tree, std::slice::from_ref(&one), "empty at source\n", &author("Second", "1700000200 +0000"));
        f.anchor(&onto, &two);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        let tip = clean(&out.result, fallback);
        let mapped = pairs(&out.mapped);
        assert_eq!(mapped.len(), 2, "fallback={fallback}");
        assert_eq!((mapped[0].0.clone(), mapped[1].0.clone()), (one, two));
        assert_eq!(mapped[1].1, tip);
        assert_ne!(mapped[1].1, mapped[0].1, "the empty source commit is kept, fallback={fallback}");
        let first = f.git.commit_info(&mapped[0].1).unwrap();
        assert_eq!(first.parents, vec![onto.clone()]);
        assert_eq!(first.tree, f.tree(&[("file", b"topic\n"), ("upstream", b"up\n")]));
        let kept = f.git.commit_info(&tip).unwrap();
        assert_eq!(kept.parents, vec![mapped[0].1.clone()]);
        assert_eq!(kept.tree, f.tree(&[("file", b"topic\n"), ("upstream", b"up\n")]));
        assert_eq!(kept.message, "empty at source\n");
        assert_eq!(kept.author, author("Second", "1700000200 +0000"));
        f.assert_state(&before);
        assert_commit_by_commit(&trace, true);
    }
}

#[test]
fn empty_range_is_clean_onto_with_no_entries() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let onto = f.commit(&f.tree(&[("file", b"onto\n")]), std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000400 +0000"));
        f.anchor(&onto, &base);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        assert_eq!(out.result, ReplayResult::Clean(onto.clone()), "fallback={fallback}");
        assert!(out.mapped.is_empty(), "fallback={fallback}");
        f.assert_state(&before);
        assert_commit_by_commit(&trace, false);
    }
}

#[test]
fn conflict_at_the_second_of_three_maps_only_the_first_and_matches_replays_conflict() {
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("log.txt", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let first_tree = f.tree(&[("log.txt", b"base\n"), ("clean", b"first replayed\n")]);
        let first = f.commit(&first_tree, std::slice::from_ref(&base), "clean first\n", &author("First", "1700000100 +0000"));
        let bad_tree = f.tree(&[("log.txt", b"topic\n"), ("clean", b"first replayed\n")]);
        let bad = f.commit(&bad_tree, std::slice::from_ref(&first), "conflict second\n", &author("Second", "1700000200 +0000"));
        let last_tree = f.tree(&[("log.txt", b"topic\n"), ("clean", b"first replayed\n"), ("later", b"must wait\n")]);
        let last = f.commit(&last_tree, std::slice::from_ref(&bad), "remaining third\n", &author("Third", "1700000300 +0000"));
        let onto = f.commit(&f.tree(&[("log.txt", b"onto\n")]), std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000400 +0000"));
        f.anchor(&onto, &last);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        assert_commit_by_commit(&trace, true);
        let replayed_tip = match out.result.clone() {
            ReplayResult::Conflicted { commit, replayed_tip, remaining, tree, files } => {
                assert_eq!(commit, bad.clone());
                assert_eq!(remaining, vec![last.clone()]);
                assert_eq!(files, vec![PathBuf::from("log.txt")]);
                let content = String::from_utf8(f.blob(&tree, "log.txt")).unwrap();
                assert!(content.contains("<<<<<<<") && content.contains("onto\n") && content.contains("topic\n"), "{content}");
                assert_eq!(f.blob(&tree, "clean"), b"first replayed\n");
                replayed_tip
            }
            other => panic!("expected named conflict, fallback={fallback}: {other:?}"),
        };
        assert_eq!(pairs(&out.mapped), vec![(first, replayed_tip.clone())], "fallback={fallback}");
        let replayed = f.git.commit_info(&replayed_tip).unwrap();
        assert_eq!(replayed.parents, vec![onto.clone()]);
        assert_eq!(replayed.message, "clean first\n");
        assert_eq!(replayed.tree, f.tree(&[("log.txt", b"onto\n"), ("clean", b"first replayed\n")]));
        match replay(&git, &base, "refs/heads/topic", &onto).unwrap() {
            ReplayResult::Conflicted { commit, replayed_tip: other_tip, remaining, files, .. } => {
                assert_eq!((commit, remaining, files), (bad, vec![last], vec![PathBuf::from("log.txt")]));
                assert_eq!(shape(&f, &other_tip, &onto), shape(&f, &replayed_tip, &onto), "fallback={fallback}");
            }
            other => panic!("replay must name the same conflict, fallback={fallback}: {other:?}"),
        }
        f.assert_state(&before);
    }
}

#[test]
fn replay_and_replay_mapped_give_the_same_chain_of_trees_messages_and_authors() {
    // Emptied, real, empty-at-source, real: replay's chosen path and replay_mapped must agree on every tree, message and author.
    for fallback in [false, true] {
        let f = Fixture::new();
        let base = f.commit(&f.tree(&[("file", b"base\n")]), &[], "base\n", &author("Base", "1700000000 +0000"));
        let onto_tree = f.tree(&[("file", b"base\n"), ("u", b"shared\n")]);
        let onto = f.commit(&onto_tree, std::slice::from_ref(&base), "onto\n", &author("Upstream", "1700000500 +0000"));
        let one = f.commit(&onto_tree, std::slice::from_ref(&base), "emptied one\n", &author("First", "1700000100 +0000"));
        let two_tree = f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("t", b"two\n")]);
        let two = f.commit(&two_tree, std::slice::from_ref(&one), "real two\n", &author("Second", "1700000200 -0530"));
        let three = f.commit(&two_tree, std::slice::from_ref(&two), "empty three\n", &author("Third", "1700000300 +0000"));
        let four_tree = f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("t", b"two\n"), ("w", b"four\n")]);
        let four = f.commit(&four_tree, std::slice::from_ref(&three), "real four\n\nMortar-Request: request-1\n", &author("Fourth", "1700000400 +0930"));
        f.anchor(&onto, &four);
        let before = f.state();
        let Some((git, trace)) = f.runner(fallback) else { continue };
        let out = replay_mapped(&git, &base, "refs/heads/topic", &onto).unwrap();
        assert_commit_by_commit(&trace, true);
        let mapped_tip = clean(&out.result, fallback);
        let replay_tip = match replay(&git, &base, "refs/heads/topic", &onto).unwrap() {
            ReplayResult::Clean(id) => id,
            other => panic!("expected clean replay, fallback={fallback}: {other:?}"),
        };
        let expected = vec![
            (f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("t", b"two\n")]), "real two\n".to_string(), author("Second", "1700000200 -0530")),
            (f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("t", b"two\n")]), "empty three\n".to_string(), author("Third", "1700000300 +0000")),
            (f.tree(&[("file", b"base\n"), ("u", b"shared\n"), ("t", b"two\n"), ("w", b"four\n")]), "real four\n\nMortar-Request: request-1\n".to_string(), author("Fourth", "1700000400 +0930")),
        ];
        assert_eq!(shape(&f, &mapped_tip, &onto), expected, "replay_mapped, fallback={fallback}");
        assert_eq!(shape(&f, &replay_tip, &onto), expected, "replay, fallback={fallback}");
        let mapped = pairs(&out.mapped);
        let sources: Vec<Oid> = mapped.iter().map(|(source, _)| source.clone()).collect();
        assert_eq!(sources, vec![one, two, three, four]);
        assert_eq!(mapped[0].1, onto);
        assert_eq!(mapped[3].1, mapped_tip);
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
    fn runner(&self, fallback: bool) -> Option<(Git, PathBuf)> {
        if !fallback && !self.git.replay_available().unwrap() {
            eprintln!("join native path unavailable; forced-fallback outcome remains exercised");
            return None;
        }
        eprintln!("join replay_mapped runner: {}", if fallback { "native replay forced unavailable" } else { "native available" });
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
fn pairs(mapped: &[MappedCommit]) -> Vec<(Oid, Oid)> { mapped.iter().map(|entry| (entry.source.clone(), entry.replayed.clone())).collect() }
fn clean(result: &ReplayResult, fallback: bool) -> Oid {
    match result {
        ReplayResult::Clean(id) => id.clone(),
        other => panic!("expected clean replay_mapped, fallback={fallback}: {other:?}"),
    }
}

/// The chain from `tip` back to `stop`, oldest first, as (tree, message, author): oids aside, what a replay gave.
fn shape(f: &Fixture, tip: &Oid, stop: &Oid) -> Vec<(Oid, String, Person)> {
    let mut chain = Vec::new();
    let mut at = tip.clone();
    while &at != stop {
        let info = f.git.commit_info(&at).unwrap();
        assert_eq!(info.parents.len(), 1, "a replayed chain is linear");
        chain.push((info.tree, info.message, info.author));
        at = info.parents[0].clone();
    }
    chain.reverse();
    chain
}

/// replay_mapped never runs git's replay under either setup; a non-empty range runs merge-tree.
fn assert_commit_by_commit(trace: &PathBuf, merged: bool) {
    let calls = fs::read_to_string(trace).unwrap_or_default();
    assert!(!calls.lines().any(|line| line == "replay"), "{calls}");
    assert_eq!(calls.lines().any(|line| line == "merge-tree"), merged, "{calls}");
}
