use crate::JoinError;
use gitio::{GitError, Oid};

const ID: &str = "1111111111111111111111111111111111111111";

fn id() -> Oid {
    Oid::parse(ID).unwrap()
}

#[test]
fn git_shows_args_code_and_stderr() {
    let error = GitError {
        args: vec!["merge-tree".to_string(), "--write-tree".to_string()],
        code: Some(128),
        stderr: "fatal: bad object\n".to_string(),
    };
    assert_eq!(JoinError::Git(error).to_string(), "git merge-tree --write-tree: exit code 128: fatal: bad object");
}

#[test]
fn onto_missing_says_attributes_cannot_be_read() {
    assert_eq!(
        JoinError::OntoMissing(id()).to_string(),
        "onto commit 1111111111111111111111111111111111111111 does not exist, so merge attributes cannot be read \
         from it"
    );
}

#[test]
fn tip_missing_names_the_full_ref() {
    assert_eq!(
        JoinError::TipMissing("refs/heads/topic".to_string()).to_string(),
        "tip ref refs/heads/topic does not exist (read by its full name)"
    );
}

#[test]
fn merge_commit_says_one_parent_only() {
    assert_eq!(
        JoinError::MergeCommit(id()).to_string(),
        "commit 1111111111111111111111111111111111111111 is a merge commit; replay takes one-parent commits only"
    );
}

#[test]
fn root_commit_says_one_parent_only() {
    assert_eq!(
        JoinError::RootCommit(id()).to_string(),
        "commit 1111111111111111111111111111111111111111 is a root commit; replay takes one-parent commits only"
    );
}

#[test]
fn tip_not_replayed_names_the_ref() {
    assert_eq!(
        JoinError::TipNotReplayed("refs/heads/topic".to_string()).to_string(),
        "git replay gave no update for the tip ref refs/heads/topic"
    );
}

#[test]
fn empty_author_date_names_the_commit_and_says_why() {
    assert_eq!(
        JoinError::EmptyAuthorDate(id()).to_string(),
        "commit 1111111111111111111111111111111111111111 has an empty author date; commit_tree cannot hand an empty \
         date back unchanged"
    );
}
