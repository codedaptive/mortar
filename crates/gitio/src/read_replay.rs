//! read_replay: read the answer of `replay --ref-action=print`.

use crate::read_message::read_message;
use crate::text_refusal::text_refusal;
use crate::{GitError, Oid, Output, RefUpdate, ReplayOutcome};

/// Exit 0: each `update <ref> <new> <old>\n` line becomes RefUpdate { refname, new, old: Some(old) }, in git's
/// order; empty output is Replayed with no updates. Exit 1 with empty stdout and empty stderr is Conflicted, the
/// shape git 2.56 gives a conflict. Exit 1 with empty stdout and something on stderr is git failing (a git without
/// the replay command, for one): returned as git's error with its stderr, never as a conflict. Exit 1 with stdout, a
/// malformed line, non-UTF-8 output or output without its final newline is refused naming what was found.
pub(crate) fn read_replay(output: &Output) -> Result<ReplayOutcome, GitError> {
    match output.status {
        0 => replay_lines(&output.stdout).map(ReplayOutcome::Replayed),
        1 if output.stdout.is_empty() && output.stderr.is_empty() => Ok(ReplayOutcome::Conflicted),
        1 if output.stdout.is_empty() => Err(GitError {
            args: Vec::new(),
            code: Some(1),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
        status => Err(GitError::refusal(
            Vec::new(),
            format!(
                "replay answered exit {status} with stdout {:?}",
                String::from_utf8_lossy(&output.stdout)
            ),
        )),
    }
}

fn replay_lines(stdout: &[u8]) -> Result<Vec<RefUpdate>, GitError> {
    let text = read_message(stdout)?;
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let body = text.strip_suffix('\n').ok_or_else(|| text_refusal("replay output without its final newline", &text))?;
    body.split('\n').map(update_line).collect()
}

/// One line, its newline already removed.
fn update_line(line: &str) -> Result<RefUpdate, GitError> {
    match line.split(' ').collect::<Vec<_>>()[..] {
        ["update", refname, new, old] if !refname.is_empty() => Ok(RefUpdate {
            refname: refname.to_string(),
            new: Oid::parse(new)?,
            old: Some(Oid::parse(old)?),
        }),
        _ => Err(text_refusal("not a replay update line", line)),
    }
}
