//! hook_command: the process for one commit hook, built but not started.

use crate::git_command::SCRUBBED;
use crate::Git;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

/// `<hook> <args>`, run directly with no shell, in `folder`. Environment, later wins: git_command's SCRUBBED variables
/// removed; then GIT_TERMINAL_PROMPT=0; then `git.env`; then GIT_EDITOR=":", which git sets for commit hooks when no
/// editor runs (as with `git commit -m`); then GIT_INDEX_FILE=`index`, last so a GIT_INDEX_FILE in `git.env` cannot
/// point the hook at another index. The prompt is off so a git the hook runs fails rather than waits on a prompt.
/// No C locale: that keeps the git messages gitio parses in English, and a hook's output is passed
/// on to a person unparsed. No optional-lock setting: the hook's own git runs are not gitio's reads.
pub(crate) fn hook_command(git: &Git, hook: &Path, args: &[OsString], folder: &Path, index: &Path) -> Command {
    let mut command = Command::new(hook);
    command.args(args).current_dir(folder);
    for key in SCRUBBED {
        command.env_remove(key);
    }
    command.env("GIT_TERMINAL_PROMPT", "0");
    command.envs(git.env.iter().map(|(k, v)| (k, v)));
    command.env("GIT_EDITOR", ":").env("GIT_INDEX_FILE", index);
    command
}
