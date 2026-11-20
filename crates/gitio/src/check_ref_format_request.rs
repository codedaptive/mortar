//! check_ref_format_request: the git run that asks whether a ref name is well formed.

use crate::no_nul::no_nul;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `check-ref-format <refname>`. check-ref-format takes neither `--end-of-options` nor `--`, so a name starting with
/// `-` would be read as an option: it is refused naming it, as is a name holding a NUL. An empty name is run (git
/// answers 1). Exit 1 (not well formed) is an answer.
pub(crate) fn check_ref_format_request(refname: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    no_nul("ref name", refname.as_bytes())?;
    if refname.starts_with('-') {
        return Err(GitError::refusal(
            Vec::new(),
            format!("not a usable ref name (starting with '-'): {refname:?}"),
        ));
    }
    Ok((
        vec![OsString::from("check-ref-format"), OsString::from(refname)],
        RunOpts {
            answer_codes: vec![1],
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
