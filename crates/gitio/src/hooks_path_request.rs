//! hooks_path_request: the git run that names the folder git runs this repository's hooks from.

use crate::RunOpts;
use std::ffi::OsString;

/// `rev-parse --path-format=absolute --git-path hooks`, read-only. `--git-path hooks` applies core.hooksPath the way
/// git does when it runs a hook: unset, the common dir's `hooks` (a linked worktree shares the main repository's);
/// an absolute value as set; `~/` expanded to $HOME; a relative value resolved against the top of the worktree git
/// runs in, or the git dir of a bare repository. `--path-format=absolute` because git prints a relative path without
/// it (the option arrived in git 2.31, below the 2.40 floor).
pub(crate) fn hooks_path_request() -> (Vec<OsString>, RunOpts) {
    (
        ["rev-parse", "--path-format=absolute", "--git-path", "hooks"].map(OsString::from).to_vec(),
        RunOpts {
            read_only: true,
            ..RunOpts::default()
        },
    )
}
