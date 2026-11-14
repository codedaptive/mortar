//! Display for JoinError: each refusal says why and names the commit or ref.

use crate::JoinError;
use std::fmt;

impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinError::Git(error) => write!(f, "{error}"),
            JoinError::OntoMissing(onto) => write!(
                f,
                "onto commit {onto} does not exist, so merge attributes cannot be read from it"
            ),
            JoinError::TipMissing(tip) => write!(f, "tip ref {tip} does not exist (read by its full name)"),
            JoinError::MergeCommit(commit) => {
                write!(f, "commit {commit} is a merge commit; replay takes one-parent commits only")
            }
            JoinError::RootCommit(commit) => {
                write!(f, "commit {commit} is a root commit; replay takes one-parent commits only")
            }
            JoinError::EmptyAuthorDate(commit) => write!(
                f,
                "commit {commit} has an empty author date; commit_tree cannot hand an empty date back unchanged"
            ),
            JoinError::TipNotReplayed(tip) => write!(f, "git replay gave no update for the tip ref {tip}"),
        }
    }
}
