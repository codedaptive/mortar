//! The folders zone's integration tests.
//! index_tree_moves: move_folder with an index tree.
//! attempts_make, attempts_keep, attempts_drop, attempts_remove, attempts_sweep (fixtures in attempt_support):
//! the attempt folder lifecycle; attempts_remake, attempts_remake_refusals and the orphan cases: remaking attempts.
//! root_files: top-level files given as folder paths.
//! zone_paths: set_zone_paths replacing a zone folder's sparse paths.
//! Real gitio worktrees, oplog transactions/locks and OS disk queries in disposable TestRepo fixtures.
#[path = "../support/mod.rs"]
mod support;
mod make;
mod repeat;
mod need;
mod guards;
mod move_support;
mod moves;
mod index_tree_support;
mod index_tree_moves;
mod maintenance_support;
mod removal;
mod sweeps;
mod sweeps_replayed;
mod disk_cases;
mod attempt_support;
mod attempts_make;
mod attempts_keep;
mod attempts_drop;
mod attempts_remove;
mod attempts_sweep;
mod attempts_remake;
mod attempts_remake_refusals;
mod root_files;
mod zone_paths;
