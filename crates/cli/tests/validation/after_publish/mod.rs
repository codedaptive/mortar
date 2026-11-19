//! After-publish across CLI, Land, Join, Folders, Snapshot, Oplog and Gitio.
//! The held-value case, acceptance step 1's check 14 and every real-crate after-publish scenario.
//! Each case owns and removes its disposable repositories, worktrees and bare remote through support::Fixture.
pub(super) mod fixture;
mod moves;
mod refusals;
mod requests;
mod guards;
