//! WO-orc-folders-it-1: SPEC-folders WO-folders-1, WO-folders-2, Tests; S-18, S-26, S-27.
//! Real gitio worktrees, oplog transactions/locks and OS disk queries in disposable TestRepo fixtures; attempt_plans
//! alone builds plain values and needs no repository.
#[path = "../support/mod.rs"]
mod support;
mod make;
mod repeat;
mod need;
mod guards;
mod move_support;
mod moves;
mod maintenance_support;
mod removal;
mod sweeps;
mod disk_cases;
mod attempt_plans;
