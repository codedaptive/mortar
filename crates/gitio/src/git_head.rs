//! Git::head: the branch a folder's HEAD names and the commit it points at. It only runs git, so it has no unit
//! tests; no gitio test calls it, and crates/snapshot/tests/snapshot/save_cases.rs reaches it through save on a
//! detached and an unborn HEAD.
//! The two answers come from two git runs, not one snapshot: a HEAD moved between them gives a branch and a commit
//! that never stood together.

use crate::head_requests::head_requests;
use crate::read_rev_parse::read_rev_parse;
use crate::read_symbolic_ref::read_symbolic_ref;
use crate::{Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// The full ref `folder`'s HEAD names (None when detached) and the commit it points at (None when unborn).
    pub fn head(&self, folder: &Path) -> Result<(Option<String>, Option<Oid>), GitError> {
        let [branch, commit] = head_requests(folder)?;
        let name = read_symbolic_ref(&self.run(&branch.0, &branch.1)?)?;
        Ok((name, read_rev_parse(&self.run(&commit.0, &commit.1)?)?))
    }
}
