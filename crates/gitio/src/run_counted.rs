//! run_counted: run git once and count the characters it writes to stdout, without keeping them.

use crate::check_exit::check_exit;
use crate::count_stream::count_stream;
use crate::git_command::git_command;
use crate::lossy_args::lossy_args;
use crate::spawn_refusal::spawn_refusal;
use crate::{Git, GitError, RunOpts};
use std::ffi::OsString;
use std::io::{self, Read};
use std::process::{ChildStderr, Stdio};

/// Runs `git -C <repo or opts.cwd> <args>` with a null stdin and counts its stdout as it arrives (see `count_stream`);
/// only stderr is kept. Git's exit is judged first, as `Git::run` judges it: an exit other than 0 or one of
/// `opts.answer_codes` is an error carrying git's stderr, whatever happened to stdout. After an accepted exit, a failed
/// stdout read is a refusal, never a short count. A git that could not be started or waited on is a refusal.
pub(crate) fn run_counted(git: &Git, args: &[OsString], opts: &RunOpts) -> Result<u64, GitError> {
    let shown = lossy_args(args);
    let refuse = |err: io::Error| spawn_refusal(&git.program, &shown, &err);
    let mut child = git_command(git, args, opts)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(refuse)?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    // stderr is drained on its own thread while this one counts stdout: git blocks once a pipe nobody reads is full,
    // so reading one pipe to its end before the other could deadlock. The counted stdout is dropped here, before the
    // wait, so a git still writing after a read error stops on a closed pipe instead of blocking.
    let (counted, stderr) = std::thread::scope(|scope| {
        let drained = scope.spawn(|| drain(stderr));
        let counted = stdout.map_or_else(|| Err(io::Error::other("no stdout pipe")), count_stream);
        (counted, drained.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic)))
    });
    let status = child.wait().map_err(refuse)?;
    check_exit(&shown, status.code(), Vec::new(), stderr, &opts.answer_codes)?;
    counted.map_err(|err| {
        GitError::refusal(shown.clone(), format!("could not read stdout of {}: {err}", git.program.display()))
    })
}

/// Everything git wrote to stderr. A read error keeps what was read: stderr only explains a failed exit, and the
/// exit itself decides the run.
fn drain(pipe: Option<ChildStderr>) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_end(&mut bytes);
    }
    bytes
}
