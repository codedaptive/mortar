//! read_ref_request: the git run that reads one full ref's id.

use crate::no_nul::no_nul;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `show-ref --verify --end-of-options <refname>`: `--verify` takes only a full ref name (never a short-name
/// lookup), and `--end-of-options` keeps a name starting with `-` from being read as an option. No `-q`: with it
/// git prints nothing on success either, so the id could not be read. Exit 128 is an answer, because git 2.56 exits
/// 128 for an absent ref; read_show_ref tells that answer from git's other errors by its stderr. An empty name, or
/// one holding a NUL, is refused naming it.
pub(crate) fn read_ref_request(refname: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    if refname.is_empty() {
        return Err(GitError::refusal(Vec::new(), format!("not a usable ref name (empty): {refname:?}")));
    }
    no_nul("ref name", refname.as_bytes())?;
    let args = ["show-ref", "--verify", "--end-of-options", refname];
    Ok((
        args.map(OsString::from).to_vec(),
        RunOpts {
            answer_codes: vec![128],
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
