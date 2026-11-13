//! trailers_parse_request: the git run that reads a message's trailers.

use crate::no_nul::no_nul;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `interpret-trailers --parse --no-divider` with the message on stdin. A `---` line does not end the message, so
/// trailers below it are read: these are commit messages, never patches (S-10). A message holding a NUL is refused
/// naming it.
pub(crate) fn trailers_parse_request(message: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    no_nul("message", message.as_bytes())?;
    Ok((
        ["interpret-trailers", "--parse", "--no-divider"].map(OsString::from).to_vec(),
        RunOpts {
            stdin: Some(message.as_bytes().to_vec()),
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
