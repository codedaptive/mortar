//! git_command: the process for one git run, built but not started.

use crate::{Git, RunOpts};
use std::ffi::OsString;
use std::process::Command;

/// Variables git sets for the hooks it runs. Mortar runs inside git hooks, and a child git that inherited them would
/// act on the hook's repository, index or object store instead of the one passed as `-C`.
const SCRUBBED: [&str; 6] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_PREFIX",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// `git.program -C <opts.cwd or git.repo> <args>`, run directly with no shell. Environment, later wins: the hook
/// variables removed; gitio's fixed settings; then `git.env`; then `opts.env`.
pub(crate) fn git_command(git: &Git, args: &[OsString], opts: &RunOpts) -> Command {
    let mut command = Command::new(&git.program);
    command
        .arg("-C")
        .arg(opts.cwd.as_deref().unwrap_or(git.repo.as_path()))
        .args(args);
    for key in SCRUBBED {
        command.env_remove(key);
    }
    // The C locale keeps git's messages in English for the parsers that read them; a prompt would hang a run that
    // has no terminal to answer it.
    command.env("LC_ALL", "C").env("GIT_TERMINAL_PROMPT", "0");
    if opts.read_only {
        // A read takes no optional locks (such as the index refresh), so it does not hold up a writer running beside it.
        command.env("GIT_OPTIONAL_LOCKS", "0");
    }
    command.envs(git.env.iter().map(|(k, v)| (k, v)));
    command.envs(opts.env.iter().map(|(k, v)| (k, v)));
    command
}
