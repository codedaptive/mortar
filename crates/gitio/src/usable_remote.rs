//! usable_remote: refuse a remote name, path or URL git could not be given safely.

use crate::no_nul::no_nul;
use crate::GitError;

/// Ok for any non-empty remote without a NUL that does not start with '-': a configured name, a path or a URL. An
/// empty remote names nothing and a NUL cannot pass through argv. A leading '-' is refused here, naming it, because
/// git 2.56 itself only stops it after starting ("fatal: strange pathname '-oops' blocked", exit 128), and an older
/// git could read it as an option.
pub(crate) fn usable_remote(remote: &str) -> Result<(), GitError> {
    if remote.is_empty() || remote.starts_with('-') {
        return Err(GitError::refusal(Vec::new(), format!("not a usable remote: {remote:?}")));
    }
    no_nul("remote", remote.as_bytes())
}
