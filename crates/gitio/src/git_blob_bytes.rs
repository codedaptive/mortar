//! Git::blob_bytes: one blob's content. It only runs git, so it has no unit tests. No test covers it yet.

use crate::blob_bytes_request::blob_bytes_request;
use crate::{Git, GitError, Oid};

impl Git {
    /// The bytes of `blob` exactly as stored (NUL bytes and a missing final newline kept). An id that names no blob
    /// (a tree, or no object) is git's error.
    pub fn blob_bytes(&self, blob: &Oid) -> Result<Vec<u8>, GitError> {
        let (args, opts) = blob_bytes_request(blob);
        Ok(self.run(&args, &opts)?.stdout)
    }
}
