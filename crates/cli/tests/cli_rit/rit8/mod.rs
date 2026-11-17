//! WO-cli-rit-8 (S-136): after-publish across CLI, Land, Join, Folders, Snapshot, Oplog and Gitio.
//! S-139/S-140, held-value R1, ACCEPT-1 #14 and every real-crate scenario in 7c's three runner headers.
//! Each case owns and removes its disposable repositories, worktrees and bare remote through support::Fixture.
mod fixture;
mod moves;
mod refusals;
mod requests;
mod guards;
