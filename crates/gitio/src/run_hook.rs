//! run_hook: run one commit hook and say whether it refused. It starts a process, so it has no unit tests;
//! tests/typed/commit_hooks.rs covers it: a refusal's exit code and separate stdout and stderr, a missing or
//! non-executable hook skipped, and the hook's argument, folder and environment.

use crate::hook_command::hook_command;
use crate::lossy_args::lossy_args;
use crate::spawn_refusal::spawn_refusal;
use crate::{Git, GitError, HookRefusal};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;

/// Runs `<hooks>/<name> <args>` as hook_command builds it, with a null stdin. Skipped (Ok(None)), as git skips it, when
/// the path is not a regular file (after following a symlink) or has no execute bit for anyone. Exit 0 is Ok(None);
/// any other exit, or death by a signal, is Ok(Some) with the exit code and the hook's stdout and stderr in full. A
/// hook that cannot be started or waited on is a refusal naming its path.
pub(crate) fn run_hook(
    git: &Git,
    hooks: &Path,
    name: &'static str,
    args: &[OsString],
    folder: &Path,
    index: &Path,
) -> Result<Option<HookRefusal>, GitError> {
    let hook = hooks.join(name);
    match std::fs::metadata(&hook) {
        Ok(meta) if meta.is_file() && meta.permissions().mode() & 0o111 != 0 => {}
        _ => return Ok(None),
    }
    let refuse = |err: std::io::Error| spawn_refusal(&hook, &lossy_args(args), &err);
    // wait_with_output drains stdout and stderr together, so a hook that fills one pipe cannot stall on it.
    let output = hook_command(git, &hook, args, folder, index)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|child| child.wait_with_output())
        .map_err(refuse)?;
    if output.status.success() {
        return Ok(None);
    }
    Ok(Some(HookRefusal {
        hook: name,
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    }))
}
