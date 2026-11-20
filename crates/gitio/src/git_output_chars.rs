//! Git::output_chars: how many characters one git read writes. It only runs git, so it has no unit tests;
//! tests/typed/output_chars.rs covers it: each query kind against plain git, paths limiting a diff, output larger than
//! one read, the folder it runs in, git's exit and the refused queries, and that DiffHead and Status leave the folder's
//! index byte-identical.

use crate::output_chars_request::output_chars_request;
use crate::run_counted::run_counted;
use crate::{BaselineQuery, Git, GitError};
use std::path::Path;

impl Git {
    /// The characters git writes to stdout for `query` run in `folder`: bytes that are not UTF-8 continuation bytes,
    /// counted as they are read and never held whole. A refused query (an empty folder, a revision that is empty,
    /// starts with '-' or holds ':', an empty path; refused before git starts) and a non-zero exit are errors, never 0.
    pub fn output_chars(&self, folder: &Path, query: &BaselineQuery) -> Result<u64, GitError> {
        let (args, opts) = output_chars_request(folder, query)?;
        run_counted(self, &args, &opts)
    }
}
