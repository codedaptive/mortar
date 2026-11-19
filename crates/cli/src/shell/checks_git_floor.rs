//! Which verbs check the installed git against gitio's floor before they run.
//!
//! Exempt: `help`, because help and `--version` are answered by entry::main_entry and never reach run; `install`
//! and `uninstall`, because they set up the harness: uninstall runs no git, and install's one git call (the toplevel
//! holding the current directory) falls back to that directory when git fails, so no git version stops either;
//! `guard-hook`, because git runs it on every ref update and an extra `git version` process there would cost every
//! update.

/// False for the exempt verbs ("help", "install", "uninstall", "guard-hook"); true for every other verb.
pub(crate) fn checks_git_floor(verb: &str) -> bool {
    !matches!(verb, "help" | "install" | "uninstall" | "guard-hook")
}
