//! `Fixture::plain`: the main repository, its base commit and its config, with no zone or request worktree and no
//! op-log recovery. For tests that run a verb needing no folder (`mr guard install`), so they skip the two worktrees
//! `Fixture::new` makes. `request` and `zone` are the main repository, so `command`, `observe` and `write` work there.
use super::{configure_main, main_repo, Fixture};

impl Fixture {
    pub fn plain() -> Self {
        let (owned, git, base) = main_repo(); let common = git.common_dir().unwrap();
        let init_args = configure_main(&git); let main = git.repo.clone();
        Self { root: owned.0.clone(), git, common, request: main.clone(), zone: main, base, init_args, _owned: owned }
    }
}
