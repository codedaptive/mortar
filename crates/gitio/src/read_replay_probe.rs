//! read_replay_probe: read whether `git replay -h` shows a replay that can print ref moves.

use crate::{GitError, Output};

/// True when the usage names `--ref-action`, which replay_request needs. The usage is read from stdout and stderr:
/// git 2.56 prints it on stdout with exit 0, older gits on stderr with exit 129. Usage without it is false, and so
/// is a git without the command ("'replay' is not a git command"). Any other error is returned unchanged.
pub(crate) fn read_replay_probe(result: Result<Output, GitError>) -> Result<bool, GitError> {
    match result {
        Ok(output) => Ok(names_ref_action(&output.stdout) || names_ref_action(&output.stderr)),
        Err(err) if err.stderr.contains("'replay' is not a git command") => Ok(false),
        Err(err) => Err(err),
    }
}

fn names_ref_action(usage: &[u8]) -> bool {
    usage.windows(b"--ref-action".len()).any(|window| window == b"--ref-action")
}
