//! Git::fetch_ref: copy one remote ref onto one local ref. It only runs git, so it has no unit tests;
//! tests/land_ops/remote_reads.rs covers it (fetch_ref forcing dst onto the remote's src, MissingSource for an absent
//! src, no tags fetched).

use crate::classify_fetch_ref::classify_fetch_ref;
use crate::fetch_ref_request::fetch_ref_request;
use crate::{FetchRefOutcome, Git, GitError};

impl Git {
    /// Sets local ref `dst` to the remote's ref `src`, forced even when that is not a fast-forward, and fetches no
    /// tags. MissingSource when the remote has no `src`, leaving `dst` untouched; any other failure is git's error.
    pub fn fetch_ref(&self, remote: &str, src: &str, dst: &str) -> Result<FetchRefOutcome, GitError> {
        let (args, opts) = fetch_ref_request(remote, src, dst)?;
        classify_fetch_ref(self.run(&args, &opts), src)
    }
}
