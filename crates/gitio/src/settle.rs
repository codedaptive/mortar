//! settle: git's exit against how feeding its stdin went.

use crate::{GitError, Output};
use std::io;
use std::path::Path;

/// Weighs git's exit against how feeding its stdin went; git's own answer wins. An exit error stands whatever
/// happened to stdin. A broken pipe after an accepted exit means git answered without reading all its input, so
/// the answer stands. Any other stdin failure is a refusal naming the write, since git did start.
pub(crate) fn settle(
    checked: Result<Output, GitError>,
    written: io::Result<()>,
    program: &Path,
    shown: &[String],
) -> Result<Output, GitError> {
    match (checked, written) {
        (Ok(_), Err(err)) if err.kind() != io::ErrorKind::BrokenPipe => Err(GitError::refusal(
            shown.to_vec(),
            format!("could not write stdin to {}: {err}", program.display()),
        )),
        (checked, _) => checked,
    }
}
