//! git_command: the process for one git run, built but not started.

use crate::no_refresh_parameters::no_refresh_parameters;
use crate::{Git, RunOpts};
use std::ffi::OsString;
use std::process::Command;

/// Variables a child git, or a commit hook gitio starts (hook_command), must not inherit. The first six are set by git
/// for the hooks it runs: Mortar runs inside git hooks, and a child that inherited them would act on the hook's
/// repository, index or object store instead of the one gitio names. The last four set a pathspec mode, which changes
/// how every pathspec gitio sends is read (under GIT_LITERAL_PATHSPECS an exclude such as `:!target/` stops
/// matching), and git refuses some pairs of them (GIT_LITERAL_PATHSPECS with GIT_GLOB_PATHSPECS or
/// GIT_ICASE_PATHSPECS; GIT_GLOB_PATHSPECS with GIT_NOGLOB_PATHSPECS).
pub(crate) const SCRUBBED: [&str; 10] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_PREFIX",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_NOGLOB_PATHSPECS",
    "GIT_LITERAL_PATHSPECS",
    "GIT_GLOB_PATHSPECS",
    "GIT_ICASE_PATHSPECS",
];

/// `git.program -C <opts.cwd or git.repo> <args>`, run directly with no shell. Environment, later wins: the hook and
/// pathspec-mode variables removed; gitio's fixed settings; then `git.env`; then `opts.env`; then, for a read run,
/// GIT_CONFIG_PARAMETERS with `'diff.autoRefreshIndex'='false'` appended to the value already in force.
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
        // GIT_OPTIONAL_LOCKS=0 stops optional locks such as status's index refresh write, so a read does not hold up
        // a writer running beside it.
        command.env("GIT_OPTIONAL_LOCKS", "0");
    }
    command.envs(git.env.iter().map(|(k, v)| (k, v)));
    command.envs(opts.env.iter().map(|(k, v)| (k, v)));
    if opts.read_only {
        // diff.autoRefreshIndex=false stops `git diff`'s stat refresh, which rewrites the index even with optional
        // locks off. It is appended to GIT_CONFIG_PARAMETERS after the caller's env so the argv stays the same for
        // every run; a caller's GIT_CONFIG_COUNT cannot undo it, since git reads GIT_CONFIG_PARAMETERS after it; and
        // a caller's or inherited value (git sets one for hooks run under `git -c`) is kept, with this entry last.
        let base = last_parameters(&opts.env)
            .or_else(|| last_parameters(&git.env))
            .or_else(|| std::env::var_os(PARAMETERS));
        command.env(PARAMETERS, no_refresh_parameters(base.as_deref()));
    }
    command
}

const PARAMETERS: &str = "GIT_CONFIG_PARAMETERS";

/// The last GIT_CONFIG_PARAMETERS value in `env`, the one that would be in force.
fn last_parameters(env: &[(OsString, OsString)]) -> Option<OsString> {
    env.iter().rev().find(|(k, _)| k == PARAMETERS).map(|(_, v)| v.clone())
}
