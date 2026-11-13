//! Git::check_ref_format: whether a ref name is well formed. It only runs git, so it has no unit tests; Nagatha's
//! integration test "check_ref_format accepting a full ref name and answering false for a malformed one" is to
//! cover it.

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
