//! trailers_add_request: the git run that adds trailers to a message.

use crate::no_nul::no_nul;
use crate::text_refusal::text_refusal;
use crate::{GitError, RunOpts};
use std::ffi::OsString;

/// `interpret-trailers --no-divider --trailer <key>=<value> ...` in the given order, with the message on stdin.
/// Refused, naming the text: a NUL in the message, a key or a value; an empty key; a key holding `=` or `:` (git
/// would split the trailer there) or a line break; a value holding a line break (it would start a line git reads on
/// its own).
pub(crate) fn trailers_add_request(
    message: &str,
    trailers: &[(String, String)],
) -> Result<(Vec<OsString>, RunOpts), GitError> {
    no_nul("message", message.as_bytes())?;
    let mut args = vec![OsString::from("interpret-trailers"), OsString::from("--no-divider")];
    for (key, value) in trailers {
        no_nul("trailer key", key.as_bytes())?;
        no_nul("trailer value", value.as_bytes())?;
        if key.is_empty() || key.contains(['=', ':', '\n', '\r']) {
            return Err(text_refusal("not a usable trailer key (empty, or holding '=', ':' or a line break)", key));
        }
        if value.contains(['\n', '\r']) {
            return Err(text_refusal("not a usable trailer value (holding a line break)", value));
        }
        args.push(OsString::from("--trailer"));
        args.push(OsString::from(format!("{key}={value}")));
    }
    Ok((
        args,
        RunOpts {
            stdin: Some(message.as_bytes().to_vec()),
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
