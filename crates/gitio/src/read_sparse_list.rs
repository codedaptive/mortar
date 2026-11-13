//! read_sparse_list: read the answer of `-c core.sparseCheckoutCone=true sparse-checkout list`.

use crate::path_from_bytes::path_from_bytes;
use crate::unquote_c_style::unquote_c_style;
use crate::{GitError, Output};
use std::path::PathBuf;

/// The cone's folders, one line each, C-quoted where git quotes them, in git's order. Empty stdout is an empty cone.
/// Anything on stderr is refused naming it: git warns there when the folder's patterns are not cone mode, and then
/// prints raw patterns, not paths. stdout without its final line break, an empty line, or a line that is not a
/// well-formed C-quoted name is refused naming stdout.
pub(crate) fn read_sparse_list(output: &Output) -> Result<Vec<PathBuf>, GitError> {
    if !output.stderr.is_empty() {
        return Err(GitError::refusal(
            Vec::new(),
            format!(
                "sparse-checkout list warned (patterns not in cone mode): {:?}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    let stdout = &output.stdout;
    if stdout.is_empty() {
        return Ok(Vec::new());
    }
    let refuse = |why: &str| {
        GitError::refusal(
            Vec::new(),
            format!("not a sparse path list ({why}): {:?}", String::from_utf8_lossy(stdout)),
        )
    };
    let body = stdout.strip_suffix(b"\n").ok_or_else(|| refuse("no final line break"))?;
    body.split(|b| *b == b'\n')
        .map(|line| {
            if line.is_empty() {
                return Err(refuse("an empty line"));
            }
            let name = unquote_c_style(line).map_err(|_| refuse("a badly quoted line"))?;
            Ok(path_from_bytes(&name))
        })
        .collect()
}
