//! replay_request: the git run that replays a branch's commits onto a new base without moving any ref.

use crate::no_nul::no_nul;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;

/// `replay --ref-action=print --onto <onto> <base>..<tip_ref>`. Without `--ref-action=print`, git 2.56 moves the ref
/// itself; with it, git only prints the moves, which the caller makes with update_refs. `tip_ref` must be a ref name:
/// given an object id, git replays and prints nothing, so there would be no answer to read. A tip that is empty,
/// holds a NUL or starts with '-' is refused. Exit 1 is the conflict answer. Not read-only: it writes commits.
pub(crate) fn replay_request(onto: &Oid, base: &Oid, tip_ref: &str) -> Result<(Vec<OsString>, RunOpts), GitError> {
    if tip_ref.is_empty() || tip_ref.starts_with('-') {
        return Err(GitError::refusal(
            Vec::new(),
            format!("not a usable tip ref (empty or starting with '-'): {tip_ref:?}"),
        ));
    }
    no_nul("tip ref", tip_ref.as_bytes())?;
    Ok((
        vec![
            "replay".into(),
            "--ref-action=print".into(),
            "--onto".into(),
            onto.as_str().into(),
            format!("{}..{tip_ref}", base.as_str()).into(),
        ],
        RunOpts {
            answer_codes: vec![1],
            ..RunOpts::default()
        },
    ))
}
