//! Git::commit_hooks: run the pre-commit and commit-msg hooks for a commit about to be made. It runs git and the
//! hooks, so it has no unit tests; tests/typed/commit_hooks.rs covers it.

use crate::next_temp_index::next_temp_index;
use crate::next_temp_message::next_temp_message;
use crate::run_hook::run_hook;
use crate::set_index_request::set_index_request;
use crate::{CommitHooksOutcome, Git, GitError, Oid};

impl Git {
    /// Runs the pre-commit hook, then the commit-msg hook, as `git commit` would for a commit of `tree` about to be
    /// made from `repo`, and answers the message to commit or the refusal.
    ///
    /// The hooks are found in the folder Git::hooks_path names (core.hooksPath, else the common dir's `hooks`) and
    /// started directly, not through `git hook run`, which merges a hook's stdout into its stderr. A hook that is
    /// missing, not a regular file or not executable is skipped, as git skips it. Each hook runs in the top of the
    /// worktree `repo` is in (a bare repository: its git dir), with a null stdin and, after gitio removes the
    /// variables git sets for its own hooks, GIT_TERMINAL_PROMPT=0, this Git value's env, GIT_EDITOR=":" and
    /// GIT_INDEX_FILE naming a temporary index read from `tree`; the folder's own index is never touched. pre-commit takes no argument.
    /// commit-msg takes one: a temporary file holding `message`, and the answer is that file's text after the hook.
    ///
    /// A hook that exits non-zero or is killed refuses the commit: the answer is Refused with the hook's name, exit
    /// code (None for a signal), stdout and stderr, and a pre-commit refusal runs no commit-msg. There is no way to
    /// skip the hooks. Nothing is written but the temporary index and message file, and both are removed on every
    /// return. Errors: a failing git run is its GitError; a hook that cannot be started, or a message file that
    /// cannot be written or read back as UTF-8, is a refusal naming it.
    pub fn commit_hooks(&self, tree: &Oid, message: &str) -> Result<CommitHooksOutcome, GitError> {
        let hooks = self.hooks_path()?;
        let folder = self.hook_folder()?;
        let index = next_temp_index();
        let (args, mut opts) = set_index_request(&folder, tree)?;
        opts.env.push(("GIT_INDEX_FILE".into(), index.0.clone().into()));
        self.run(&args, &opts)?;
        if let Some(refusal) = run_hook(self, &hooks, "pre-commit", &[], &folder, &index.0)? {
            return Ok(CommitHooksOutcome::Refused(refusal));
        }
        let file = next_temp_message();
        std::fs::write(&file.0, message).map_err(|err| {
            let shown = file.0.display();
            GitError::refusal(Vec::new(), format!("could not write the commit message to {shown}: {err}"))
        })?;
        let msg_args = [file.0.clone().into_os_string()];
        if let Some(refusal) = run_hook(self, &hooks, "commit-msg", &msg_args, &folder, &index.0)? {
            return Ok(CommitHooksOutcome::Refused(refusal));
        }
        let text = std::fs::read_to_string(&file.0).map_err(|err| {
            let shown = file.0.display();
            GitError::refusal(Vec::new(), format!("could not read the message commit-msg left in {shown}: {err}"))
        })?;
        Ok(CommitHooksOutcome::Message(text))
    }
}
