//! Git::update_refs: move several refs at once, all or none. It only runs git, so it has no unit tests;
//! tests/update_refs.rs covers it: a clean move of several refs, and a stale expected value answering Stale naming
//! the ref with nothing moved.

use crate::classify_update_refs::classify_update_refs;
use crate::update_refs_stdin::update_refs_stdin;
use crate::{Git, GitError, RefUpdate, RunOpts, UpdateRefsOutcome};

impl Git {
    /// Moves every ref in `updates` or none of them, each only if it holds its expected old value. Returns Stale
    /// naming the first refused ref in update order; any other failure is returned as a GitError.
    pub fn update_refs(&self, updates: &[RefUpdate]) -> Result<UpdateRefsOutcome, GitError> {
        let stdin = update_refs_stdin(updates)?;
        classify_update_refs(
            self.run(
                &["update-ref", "--stdin", "-z"],
                &RunOpts {
                    stdin: Some(stdin),
                    ..RunOpts::default()
                },
            ),
            updates,
        )
    }
}
