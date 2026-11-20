//! read_show_ref: read the answer of `show-ref --verify` for one ref.

use crate::{GitError, Oid, Output};

/// Exit 0 is exactly `<oid> <refname>\n`, the name being the one sent (a symbolic ref prints its target's id under
/// the name asked): Some(oid). Exit 128 with nothing on stdout and stderr exactly
/// `fatal: '<refname>' - not a valid ref\n` is None (absent). Any other exit 128 is git's own error (not a repo, for
/// one), returned with its stderr and never as None. Anything else is refused naming what was found.
pub(crate) fn read_show_ref(output: &Output, refname: &str) -> Result<Option<Oid>, GitError> {
    let absent = format!("fatal: '{refname}' - not a valid ref\n");
    match output.status {
        0 => found(&output.stdout, refname).map(Some),
        128 if output.stdout.is_empty() && output.stderr == absent.as_bytes() => Ok(None),
        128 => Err(GitError {
            args: Vec::new(),
            code: Some(128),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
        status => Err(GitError::refusal(
            Vec::new(),
            format!(
                "show-ref answered exit {status} with stdout {:?}",
                String::from_utf8_lossy(&output.stdout)
            ),
        )),
    }
}

/// The id from the one line `<oid> <refname>\n`; any other text is refused naming it.
fn found(stdout: &[u8], refname: &str) -> Result<Oid, GitError> {
    let text = String::from_utf8_lossy(stdout);
    text.strip_suffix('\n')
        .and_then(|line| line.split_once(' '))
        .filter(|(_, name)| *name == refname)
        .and_then(|(hex, _)| Oid::parse(hex).ok())
        .ok_or_else(|| {
            GitError::refusal(
                Vec::new(),
                format!("not one show-ref line for {refname:?}: {text:?}"),
            )
        })
}
