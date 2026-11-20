//! Git::check_ref_format: whether a ref name is well formed. It only runs git, so it has no unit tests; oplog's
//! tests/oplog/ref_names.rs (apply_refuses_a_name_git_check_ref_format_rejects_before_writing_an_intent) covers it:
//! apply refuses a name git rejects, as BadMove, before writing an intent.

use crate::check_ref_format_request::check_ref_format_request;
use crate::read_check_ref_format::read_check_ref_format;
use crate::{Git, GitError};

impl Git {
    /// True when `refname` is a well-formed ref name, false when git says it is not.
    pub fn check_ref_format(&self, refname: &str) -> Result<bool, GitError> {
        let (args, opts) = check_ref_format_request(refname)?;
        read_check_ref_format(&self.run(&args, &opts)?)
    }
}
