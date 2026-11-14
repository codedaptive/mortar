//! resolved_folder_tree: reads the old request tip, then builds its tree with every round's paths from the caller's
//! tree; a refusal naming an earlier round's path that became a folder is ConflictedPathIsFolder naming it.

use crate::fake_call::FakeCall;
use crate::join_state::JoinState;
use crate::replay_git_fake::ReplayGitFake;
use crate::resolved_folder_tree::resolved_folder_tree;
use crate::JoinRunError;
use gitio::{CommitInfo, Oid, Person, TreeWithPathsError, TreeWithPathsRefusal};
use std::path::PathBuf;

const REQUEST_TIP: &str = "1111111111111111111111111111111111111111";
const OLD_TIP_TREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CALLER_TREE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// A state in its second round: `files` is this round's, `all_files` holds an earlier round's `old/c.md` too. Its
/// conflicted `tree` differs from the old tip's tree, which is the base resolved_folder_tree must use.
fn state() -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid(REQUEST_TIP),
        onto: oid("2222222222222222222222222222222222222222"),
        replayed_tip: oid("3333333333333333333333333333333333333333"),
        commit: oid("4444444444444444444444444444444444444444"),
        tree: oid("5555555555555555555555555555555555555555"),
        remaining: Vec::new(),
        files: vec![PathBuf::from("src/a.rs")],
        all_files: vec![PathBuf::from("old/c.md"), PathBuf::from("src/a.rs")],
        files_written: true,
        rebased_to: None,
        folder_tree: None,
    }
}

fn fake(tree_with_paths: Result<Oid, TreeWithPathsError>) -> ReplayGitFake {
    let info = CommitInfo {
        tree: oid(OLD_TIP_TREE),
        parents: vec![oid("6666666666666666666666666666666666666666")],
        author: Person { name: "Ana".to_string(), email: "ana@example.com".to_string(), date: None },
        message: "request change\n".to_string(),
    };
    let mut git = ReplayGitFake { tree_with_paths: Some(tree_with_paths), ..ReplayGitFake::default() };
    git.commits.insert(oid(REQUEST_TIP), info);
    git
}

#[test]
fn builds_the_old_tips_tree_with_every_rounds_paths_from_the_callers_tree() {
    let git = fake(Ok(oid("cccccccccccccccccccccccccccccccccccccccc")));
    let answer = resolved_folder_tree(&git, &state(), &oid(CALLER_TREE));
    assert!(
        matches!(&answer, Ok(tree) if *tree == oid("cccccccccccccccccccccccccccccccccccccccc")),
        "{answer:?}"
    );
    assert_eq!(
        *git.calls.borrow(),
        vec![
            FakeCall::CommitInfo(oid("1111111111111111111111111111111111111111")),
            FakeCall::TreeWithPaths {
                base: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                from: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                paths: vec![PathBuf::from("old/c.md"), PathBuf::from("src/a.rs")],
            },
        ]
    );
}

#[test]
fn an_earlier_rounds_path_that_became_a_folder_is_refused_naming_it() {
    let refusal = TreeWithPathsError::Refused(TreeWithPathsRefusal::FolderInFrom { path: PathBuf::from("old/c.md") });
    let answer = resolved_folder_tree(&fake(Err(refusal)), &state(), &oid(CALLER_TREE));
    let Err(JoinRunError::ConflictedPathIsFolder { path }) = answer else { panic!("{answer:?}") };
    assert_eq!(path, PathBuf::from("old/c.md"));
}
