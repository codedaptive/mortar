//! Git::range_diff_ranges: what changed between two commit ranges. It only runs git, so it has no unit tests; its
//! zone integration tests are in tests/range_diff_ranges.rs.

use crate::range_diff_ranges_request::range_diff_ranges_request;
use crate::{Git, GitError, Oid};

impl Git {
    /// git's range-diff of `base1..tip1` against `base2..tip2`, for comparing a series remade on a moved base with
    /// the reviewed one, as text for a reviewer: bytes that are not UTF-8 show as U+FFFD. Every argument is an Oid,
    /// so none can start with `-` or carry an extra `..`.
    pub fn range_diff_ranges(
        &self,
        base1: &Oid,
        tip1: &Oid,
        base2: &Oid,
        tip2: &Oid,
    ) -> Result<String, GitError> {
        let (args, opts) = range_diff_ranges_request(base1, tip1, base2, tip2);
        Ok(String::from_utf8_lossy(&self.run(&args, &opts)?.stdout).into_owned())
    }
}
