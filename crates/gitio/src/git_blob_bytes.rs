//! Git::blob_bytes: one blob's content. It only runs git, so it has no unit tests; Nagatha's integration test
//! "blob_bytes reading a blob holding NUL bytes unchanged; a tree id is git's error" is to cover it.

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
