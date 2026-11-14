//! guard_hook_script: the text of a hook the guard writes.

/// The one-line shell script for hook `name`. `exec` hands the hook's stdin and arguments (the transaction state,
/// or the remote's name and url) straight to `mortar guard-hook`, which decides.
pub(crate) fn guard_hook_script(name: &str) -> String {
    format!("#!/bin/sh\nexec mortar guard-hook {name} \"$@\"\n")
}
