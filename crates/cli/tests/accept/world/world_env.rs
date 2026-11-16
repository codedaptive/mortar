//! The whole environment of every git and mortar child the drive runs.

use std::ffi::{OsStr, OsString};
use std::path::Path;

/// PATH as given; HOME and TMPDIR the throwaway root; no global or system git config (a global core.hooksPath
/// would otherwise leak in); C locale and no terminal prompts, so output is plain; a fixed author and committer.
/// Nothing else: callers env_clear first, so nothing of the caller's environment reaches the child.
pub fn world_env(root: &Path, path: &OsStr) -> Vec<(OsString, OsString)> {
    let fixed = [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("LC_ALL", "C"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("GIT_AUTHOR_NAME", "Accept Drive"),
        ("GIT_AUTHOR_EMAIL", "accept@example.invalid"),
        ("GIT_COMMITTER_NAME", "Accept Drive"),
        ("GIT_COMMITTER_EMAIL", "accept@example.invalid"),
    ];
    let mut env = vec![
        (OsString::from("PATH"), path.to_os_string()),
        (OsString::from("HOME"), root.as_os_str().to_os_string()),
        (OsString::from("TMPDIR"), root.as_os_str().to_os_string()),
    ];
    env.extend(fixed.iter().map(|(key, value)| (OsString::from(key), OsString::from(value))));
    env
}
