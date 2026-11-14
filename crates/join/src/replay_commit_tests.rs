use crate::fake_call::FakeCall;
use crate::replay_commit::replay_commit;
use crate::replay_git_fake::ReplayGitFake;
use crate::step::Step;
use crate::JoinError;
use gitio::{CommitInfo, MergeTreeOutcome, Oid, Person};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;

const ONTO: &str = "1111111111111111111111111111111111111111";
const RUNNING: &str = "2222222222222222222222222222222222222222";
const COMMIT: &str = "3333333333333333333333333333333333333333";
const PARENT: &str = "4444444444444444444444444444444444444444";
const OTHER_PARENT: &str = "5555555555555555555555555555555555555555";
const TREE: &str = "6666666666666666666666666666666666666666";
const NEW: &str = "7777777777777777777777777777777777777777";
const OLD_TREE: &str = "8888888888888888888888888888888888888888";
const MESSAGE: &str = "Fix the parser\n\nMortar-Change-Id: k3x9q2\nMortar-Zone: join\n";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn author() -> Person {
    Person { name: "Ana Núñez".to_string(), email: "ana@example.com".to_string(), date: Some("1700000000 +0530".to_string()) }
}

fn fake(parents: &[&str], merge: Option<MergeTreeOutcome>) -> ReplayGitFake {
    let info = CommitInfo {
        tree: oid(OLD_TREE),
        parents: parents.iter().map(|hex| oid(hex)).collect(),
        author: author(),
        message: MESSAGE.to_string(),
    };
    let mut git = ReplayGitFake { new_commits: RefCell::new(VecDeque::from([oid(NEW)])), ..ReplayGitFake::default() };
    git.commits.insert(oid(COMMIT), info);
    if let Some(merge) = merge {
        git.merges.insert(oid(COMMIT), merge);
    }
    git
}

fn merge_call() -> FakeCall {
    FakeCall::MergeTree { attr_source: oid(ONTO), merge_base: Some(oid(PARENT)), ours: oid(RUNNING), theirs: oid(COMMIT) }
}

#[test]
fn clean_merges_from_the_parent_and_commits_keeping_message_and_author() {
    let git = fake(&[PARENT], Some(MergeTreeOutcome::Clean(oid(TREE))));
    assert_eq!(replay_commit(&git, &oid(ONTO), &oid(RUNNING), &oid(COMMIT)), Ok(Step::Clean(oid(NEW))));
    assert_eq!(
        *git.calls.borrow(),
        vec![
            FakeCall::CommitInfo(oid(COMMIT)),
            merge_call(),
            FakeCall::CommitTree {
                tree: oid(TREE),
                parents: vec![oid(RUNNING)],
                message: "Fix the parser\n\nMortar-Change-Id: k3x9q2\nMortar-Zone: join\n".to_string(),
                author: Some(Person {
                    name: "Ana Núñez".to_string(),
                    email: "ana@example.com".to_string(),
                    date: Some("1700000000 +0530".to_string()),
                }),
                committer: None,
            },
        ]
    );
}

#[test]
fn conflicted_gives_tree_and_files_and_writes_no_commit() {
    let files = vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")];
    let git = fake(&[PARENT], Some(MergeTreeOutcome::Conflicted { tree: oid(TREE), files }));
    assert_eq!(
        replay_commit(&git, &oid(ONTO), &oid(RUNNING), &oid(COMMIT)),
        Ok(Step::Conflicted { tree: oid(TREE), files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")] })
    );
    assert_eq!(*git.calls.borrow(), vec![FakeCall::CommitInfo(oid(COMMIT)), merge_call()]);
}

#[test]
fn root_commit_is_refused_before_any_merge() {
    let git = fake(&[], None);
    assert_eq!(
        replay_commit(&git, &oid(ONTO), &oid(RUNNING), &oid(COMMIT)),
        Err(JoinError::RootCommit(oid(COMMIT)))
    );
    assert_eq!(*git.calls.borrow(), vec![FakeCall::CommitInfo(oid(COMMIT))]);
}

#[test]
fn merge_commit_is_refused_before_any_merge() {
    let git = fake(&[PARENT, OTHER_PARENT], None);
    assert_eq!(
        replay_commit(&git, &oid(ONTO), &oid(RUNNING), &oid(COMMIT)),
        Err(JoinError::MergeCommit(oid(COMMIT)))
    );
    assert_eq!(*git.calls.borrow(), vec![FakeCall::CommitInfo(oid(COMMIT))]);
}
