//! Git::ls_remote: list a remote's refs. It only runs git, so it has no unit tests; tests/land_ops/remote_reads.rs
//! covers it (ls_remote listing a pattern's refs, tail-matched, a tag with and without its peeled line, empty for no
//! match).

use crate::ls_remote_request::ls_remote_request;
use crate::read_ls_remote::read_ls_remote;
use crate::{Git, GitError, Oid};

impl Git {
    /// The remote's refs matching `patterns` as (oid, refname), in git's order; every ref, HEAD included, when
    /// `patterns` is empty. Git matches a pattern against the tail of a refname by whole path parts (`main` lists
    /// `refs/heads/main`); an annotated tag's peeled `<refname>^{}` line is listed when a pattern matches that name
    /// too (no pattern, or a glob such as `tags/*`; the exact `v1` lists the tag alone). Empty when nothing matches.
    pub fn ls_remote(&self, remote: &str, patterns: &[String]) -> Result<Vec<(Oid, String)>, GitError> {
        let (args, opts) = ls_remote_request(remote, patterns)?;
        read_ls_remote(&self.run(&args, &opts)?.stdout)
    }
}
