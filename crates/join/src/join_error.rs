//! JoinError: why a join step did not happen. A declaration only; join_error_display_tests and
//! join_error_from_git_tests exercise it, and check_onto_tests, read_git_replay_tests, replay_commit_tests and
//! replay_in_tests return each refusal.

use gitio::{GitError, Oid};

/// A git run that failed, or a refusal made before or after running git.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JoinError {
    /// A git run failed.
    Git(GitError),
    /// The onto commit does not exist, so merge attributes cannot be read from it.
    OntoMissing(Oid),
    /// The tip's full ref name is absent.
    TipMissing(String),
    /// The commit has more than one parent; replay takes one-parent commits only.
    MergeCommit(Oid),
    /// The commit has no parent; replay takes one-parent commits only.
    RootCommit(Oid),
    /// git's replay was clean but gave no update for the tip ref.
    TipNotReplayed(String),
}
