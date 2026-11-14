//! Git::remote_push_urls: a remote's push URLs as git resolves them. It only runs git, so it has no unit tests;
//! tests/remote_push_urls covers it.

use crate::read_remote_push_urls::read_remote_push_urls;
use crate::remote_push_urls_request::remote_push_urls_request;
use crate::{Git, GitError};

impl Git {
    /// The URLs git would push `remote` to, exactly as git prints them: its `pushurl` values in config order when any
    /// is set, else its `url`, with `insteadOf` and `pushInsteadOf` rewrites applied by git. A remote with no url
    /// answers its own name (git's behaviour, kept), so Ok is never empty for an existing remote: a caller must not
    /// read an empty Vec as "no URLs, allow". An unknown remote, or a URL or path given as the remote, is an Err with
    /// code Some(2), git's exit, recognisable without reading stderr. A config git cannot read is an Err with git's
    /// code (128 for a bad config line). A name git cannot be given (empty, a leading '-', a NUL) and a URL that is
    /// not UTF-8 are refusals with code None.
    pub fn remote_push_urls(&self, remote: &str) -> Result<Vec<String>, GitError> {
        let (args, opts) = remote_push_urls_request(remote)?;
        read_remote_push_urls(&self.run(&args, &opts)?.stdout)
    }
}
