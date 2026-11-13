//! Git::run: run git once and collect what it wrote.

use crate::check_exit::check_exit;
use crate::git_command::git_command;
use crate::lossy_args::lossy_args;
use crate::settle::settle;
use crate::spawn_refusal::spawn_refusal;
use crate::{Git, GitError, Output, RunOpts};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::process::{ChildStdin, Stdio};

impl Git {
    /// Runs `git -C <repo or opts.cwd> <args>` and waits for it. Ok for exit 0 or one of `opts.answer_codes`; any other exit
    /// is an error carrying git's stderr. A git that could not be started or waited on is a refusal, and so is a
    /// failure to feed its stdin, unless git's exit already decided the run (see `settle`).
    pub fn run<S: AsRef<OsStr>>(&self, args: &[S], opts: &RunOpts) -> Result<Output, GitError> {
        let args: Vec<OsString> = args.iter().map(|arg| arg.as_ref().to_os_string()).collect();
        let shown = lossy_args(&args);
        let refuse = |err: io::Error| spawn_refusal(&self.program, &shown, &err);
        let mut child = git_command(self, &args, opts)
            .stdin(match opts.stdin {
                Some(_) => Stdio::piped(),
                None => Stdio::null(),
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(refuse)?;
        let pipe = child.stdin.take();
        // stdin is fed on its own thread while this one drains stdout and stderr: git may fill an output pipe before
        // it has read all its input, and a single thread doing both in turn would deadlock.
        let (written, output) = std::thread::scope(|scope| {
            let writer = scope.spawn(|| feed(pipe, opts.stdin.as_deref()));
            let output = child.wait_with_output();
            (writer.join(), output)
        });
        let output = output.map_err(refuse)?;
        settle(
            check_exit(&shown, output.status.code(), output.stdout, output.stderr, &opts.answer_codes),
            written.unwrap_or_else(|panic| std::panic::resume_unwind(panic)),
            &self.program,
            &shown,
        )
    }
}

/// Writes `bytes` to git's stdin, then drops the pipe so git sees end of input.
fn feed(pipe: Option<ChildStdin>, bytes: Option<&[u8]>) -> io::Result<()> {
    match (pipe, bytes) {
        (Some(mut pipe), Some(bytes)) => pipe.write_all(bytes),
        _ => Ok(()),
    }
}
