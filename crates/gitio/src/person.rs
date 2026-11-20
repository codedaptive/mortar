//! Person: who wrote or committed a commit. A declaration only; commit_tree_request_tests exercises it.

/// A commit's author or committer, as git's identity environment takes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub name: String,
    pub email: String,
    /// A date in any form git accepts (such as `1700000000 +0000`); None lets git use the current time.
    pub date: Option<String>,
}
