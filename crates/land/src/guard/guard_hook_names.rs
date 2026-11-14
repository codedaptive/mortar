//! The two hooks the guard installs: reference-transaction stops local moves, pre-push stops pushes. A const only
//! (S-7); tests/guard_zit.rs fresh_install_writes_config_and_hooks and install_follows_a_relative_hooks_path assert
//! both hook files by name.

/// The hook file names, in the order guard_install checks them and names the first foreign one.
pub(crate) const GUARD_HOOK_NAMES: [&str; 2] = ["reference-transaction", "pre-push"];
