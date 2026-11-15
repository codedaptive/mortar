use crate::fake_call::FakeCall;
use crate::replay_fallback::replay_fallback;
use crate::replay_git_fake::ReplayGitFake;
use crate::{MappedCommit, MappedReplay, ReplayResult};
use gitio::{CommitInfo, MergeTreeOutcome, Oid, Person};
use std::cell::RefCell;
use std::path::PathBuf;

const BASE: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";
const ONTO: &str = "3333333333333333333333333333333333333333";
const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const NEW_A: &str = "a000000000000000000000000000000000000001";
const NEW_B: &str = "b000000000000000000000000000000000000002";
const NEW_C: &str = "c000000000000000000000000000000000000003";
const TREE_A: &str = "a111111111111111111111111111111111111111";
const TREE_B: &str = "b111111111111111111111111111111111111111";
const TREE_C: &str = "c111111111111111111111111111111111111111";
const RUNNING_TREE: &str = "9999999999999999999999999999999999999999";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn author() -> Person {
    Person { name: "Bo".to_string(), email: "bo@example.com".to_string(), date: Some("1700000000 +0000".to_string()) }
}

/// A fake holding A, B and C, each the parent of the next (A's parent is BASE), with messages "a\n", "b\n" and "c\n",
/// and the running tips ONTO, NEW_A and NEW_B with tree RUNNING_TREE, which no merge gives, so no commit drops.
fn fake(merges: [(&str, MergeTreeOutcome); 3], new_commits: &[&str]) -> ReplayGitFake {
    let mut git = ReplayGitFake {
        rev_list: Some(vec![oid(A), oid(B), oid(C)]),
        new_commits: RefCell::new(new_commits.iter().map(|hex| oid(hex)).collect()),
        ..ReplayGitFake::default()
    };
    for (commit, parent, message) in [(A, BASE, "a\n"), (B, A, "b\n"), (C, B, "c\n")] {
        let info = CommitInfo { tree: oid(TIP), parents: vec![oid(parent)], author: author(), message: message.into() };
        git.commits.insert(oid(commit), info);
    }
    for running in [ONTO, NEW_A, NEW_B] {
        let info = CommitInfo { tree: oid(RUNNING_TREE), parents: Vec::new(), author: author(), message: "".into() };
        git.commits.insert(oid(running), info);
    }
    for (commit, merge) in merges {
        git.merges.insert(oid(commit), merge);
    }
    git
}

fn merge(theirs: &str, parent: &str, ours: &str) -> FakeCall {
    FakeCall::MergeTree { attr_source: oid(ONTO), merge_base: Some(oid(parent)), ours: oid(ours), theirs: oid(theirs) }
}

fn commit(tree: &str, parent: &str, message: &str) -> FakeCall {
    FakeCall::CommitTree {
        tree: oid(tree),
        parents: vec![oid(parent)],
        message: message.to_string(),
        author: Some(author()),
        committer: None,
    }
}

#[test]
fn empty_range_is_clean_at_onto() {
    let git = ReplayGitFake { rev_list: Some(Vec::new()), ..ReplayGitFake::default() };
    assert_eq!(
        replay_fallback(&git, &oid(BASE), &oid(TIP), &oid(ONTO)),
        Ok(MappedReplay { result: ReplayResult::Clean(oid(ONTO)), mapped: Vec::new() })
    );
    assert_eq!(*git.calls.borrow(), vec![FakeCall::RevList { base: oid(BASE), tip: oid(TIP) }]);
}

#[test]
fn three_clean_commits_chain_each_onto_the_last_new_commit() {
    let merges = [
        (A, MergeTreeOutcome::Clean(oid(TREE_A))),
        (B, MergeTreeOutcome::Clean(oid(TREE_B))),
        (C, MergeTreeOutcome::Clean(oid(TREE_C))),
    ];
    let git = fake(merges, &[NEW_A, NEW_B, NEW_C]);
    assert_eq!(
        replay_fallback(&git, &oid(BASE), &oid(TIP), &oid(ONTO)),
        Ok(MappedReplay {
            result: ReplayResult::Clean(oid(NEW_C)),
            mapped: vec![
                MappedCommit { source: oid(A), replayed: oid(NEW_A) },
                MappedCommit { source: oid(B), replayed: oid(NEW_B) },
                MappedCommit { source: oid(C), replayed: oid(NEW_C) },
            ],
        })
    );
    assert_eq!(
        *git.calls.borrow(),
        vec![
            FakeCall::RevList { base: oid(BASE), tip: oid(TIP) },
            FakeCall::CommitInfo(oid(A)),
            merge(A, BASE, ONTO),
            FakeCall::CommitInfo(oid(ONTO)),
            commit(TREE_A, ONTO, "a\n"),
            FakeCall::CommitInfo(oid(B)),
            merge(B, A, NEW_A),
            FakeCall::CommitInfo(oid(NEW_A)),
            commit(TREE_B, NEW_A, "b\n"),
            FakeCall::CommitInfo(oid(C)),
            merge(C, B, NEW_B),
            FakeCall::CommitInfo(oid(NEW_B)),
            commit(TREE_C, NEW_B, "c\n"),
        ]
    );
}

#[test]
fn conflict_on_the_second_stops_with_the_state_and_never_touches_the_third() {
    let files = vec![PathBuf::from("x.txt")];
    let merges = [
        (A, MergeTreeOutcome::Clean(oid(TREE_A))),
        (B, MergeTreeOutcome::Conflicted { tree: oid(TREE_B), files }),
        (C, MergeTreeOutcome::Clean(oid(TREE_C))),
    ];
    let git = fake(merges, &[NEW_A]);
    assert_eq!(
        replay_fallback(&git, &oid(BASE), &oid(TIP), &oid(ONTO)),
        Ok(MappedReplay {
            result: ReplayResult::Conflicted {
                commit: oid(B),
                replayed_tip: oid(NEW_A),
                remaining: vec![oid(C)],
                tree: oid(TREE_B),
                files: vec![PathBuf::from("x.txt")],
            },
            mapped: vec![MappedCommit { source: oid(A), replayed: oid(NEW_A) }],
        })
    );
    assert_eq!(
        *git.calls.borrow(),
        vec![
            FakeCall::RevList { base: oid(BASE), tip: oid(TIP) },
            FakeCall::CommitInfo(oid(A)),
            merge(A, BASE, ONTO),
            FakeCall::CommitInfo(oid(ONTO)),
            commit(TREE_A, ONTO, "a\n"),
            FakeCall::CommitInfo(oid(B)),
            merge(B, A, NEW_A),
        ]
    );
}
