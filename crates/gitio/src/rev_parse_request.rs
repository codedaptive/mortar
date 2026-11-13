//! rev_parse_request: the git run that resolves a rev to a commit.

use crate::no_nul::no_nul;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `rev-parse --verify -q --end-of-options <rev>^{commit}`: `--end-of-options` keeps a rev starting with `-` from
/// being read as an option, and `^{commit}` peels a tag and refuses a non-commit. Exit 1 (not found) is an answer.
/// An empty rev, or one holding a NUL, is refused naming it.
pub(crate) fn rev_parse_request(rev: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    if rev.is_empty() {
        return Err(GitError::refusal(Vec::new(), format!("not a usable rev (empty): {rev:?}")));
    }
    no_nul("rev", rev.as_bytes())?;
    let args = ["rev-parse", "--verify", "-q", "--end-of-options", &format!("{rev}^{{commit}}")];
    Ok((
        args.map(OsString::from).to_vec(),
        RunOpts {
            answer_codes: vec![1],
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
