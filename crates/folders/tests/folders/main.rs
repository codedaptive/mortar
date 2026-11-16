//! WO-orc-folders-it-1: SPEC-folders WO-folders-1, WO-folders-2, Tests; S-18, S-26, S-27.
//! index_tree_moves: WO-folders-4c, S-87, S-88.
//! attempts_make, attempts_keep, attempts_drop, attempts_remove, attempts_sweep (fixtures in attempt_support):
//! WO-folders-6c, S-69, S-82, S-93; attempts_remake, attempts_remake_refusals and the orphan cases: WO-folders-6d,
//! S-96 (2), S-98.
//! root_files: WO-folders-sparse-files, S-126.
//! Real gitio worktrees, oplog transactions/locks and OS disk queries in disposable TestRepo fixtures; attempt_plans,
//! sweep_pick and sweep_pick_replayed alone build plain values and need no repository (ref_pairs holds their shared
//! conversions).
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
mod sweep_pick_replayed;
mod sweep_pick;
mod ref_pairs;
mod disk_cases;
mod attempt_plans;
mod attempt_support;
mod attempts_make;
mod attempts_keep;
mod attempts_drop;
mod attempts_remove;
mod attempts_sweep;
mod attempts_remake;
mod attempts_remake_refusals;
mod root_files;
