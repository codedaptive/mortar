//! WO-cli-rit-11: CLI undo/save/status/join/resolve with real Snapshot, Join, Folders, Oplog and Gitio.
//! S-142 own attempt refs and repeat safety; S-143 pending moves and refusal-before-finish ordering.
//! Every case owns its disposable repository/worktrees/remote through the reused support::Fixture.
mod fixture;
mod saves;
mod rebases;
mod refusals;
mod guards;
mod killed;
