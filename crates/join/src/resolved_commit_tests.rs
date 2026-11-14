//! resolved_commit: commits the resolved tree on the replayed tip with the conflicted commit's message and author;
//! an empty author date is refused before any commit is written.

use crate::fake_call::FakeCall;
use crate::join_state::JoinState;
use crate::replay_git_fake::ReplayGitFake;
use crate::resolved_commit::resolved_commit;
use crate::JoinError;
use gitio::{CommitInfo, Oid, Person};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;

const COMMIT: &str = "4444444444444444444444444444444444444444";
const NEW: &str = "7777777777777777777777777777777777777777";
const MESSAGE: &str = "Fix the parser\n\nMortar-Change-Id: k3x9q2\nMortar-Zone: join\n";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn author(date: &str) -> Person {
    Person { name: "Ana Núñez".to_string(), email: "ana@example.com".to_string(), date: Some(date.to_string()) }
}

fn state() -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid("1111111111111111111111111111111111111111"),
        onto: oid("2222222222222222222222222222222222222222"),
        replayed_tip: oid("3333333333333333333333333333333333333333"),
        commit: oid(COMMIT),
        tree: oid("5555555555555555555555555555555555555555"),
        remaining: Vec::new(),
        files: vec![PathBuf::from("x.txt")],
        files_written: true,
        rebased_to: None,
    }
}

/// The conflicted commit's info: its own old tree and parent, which the resolved commit must not reuse.
fn fake(date: &str) -> ReplayGitFake {
    let info = CommitInfo {
        tree: oid("8888888888888888888888888888888888888888"),
        parents: vec![oid("9999999999999999999999999999999999999999")],
        author: author(date),
        message: MESSAGE.to_string(),
    };
    let mut git = ReplayGitFake { new_commits: RefCell::new(VecDeque::from([oid(NEW)])), ..ReplayGitFake::default() };
    git.commits.insert(oid(COMMIT), info);
    git
}

#[test]
fn commits_the_resolved_tree_on_the_replayed_tip_with_message_and_author() {
    let git = fake("1700000000 +0530");
    let tree = oid("6666666666666666666666666666666666666666");
    assert_eq!(resolved_commit(&git, &state(), &tree), Ok(oid(NEW)));
    assert_eq!(
        *git.calls.borrow(),
        vec![
            FakeCall::CommitInfo(oid("4444444444444444444444444444444444444444")),
            FakeCall::CommitTree {
                tree: oid("6666666666666666666666666666666666666666"),
                parents: vec![oid("3333333333333333333333333333333333333333")],
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
fn empty_author_date_is_refused_with_no_commit_written() {
    let git = fake("");
    let tree = oid("6666666666666666666666666666666666666666");
    assert_eq!(
        resolved_commit(&git, &state(), &tree),
        Err(JoinError::EmptyAuthorDate(oid("4444444444444444444444444444444444444444")))
    );
    assert_eq!(*git.calls.borrow(), vec![FakeCall::CommitInfo(oid(COMMIT))]);
}
