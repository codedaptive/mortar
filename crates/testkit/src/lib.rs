//! Shared test-only repository and child-process support.
//! Consumers use this crate only through dev-dependencies; no production crate is linked here.

mod run_with_kill_gate;
mod test_child;
mod write_git_kill_shim;
mod test_repo;
mod test_repo_close;
mod test_repo_new;
mod write_git_shim;

pub use run_with_kill_gate::run_with_kill_gate;
pub use test_child::test_child;
pub use write_git_kill_shim::write_git_kill_shim;
pub use test_repo::TestRepo;
pub use write_git_shim::write_git_shim;
