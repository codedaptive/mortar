//! CLI undo/save/status/join/resolve with real Snapshot, Join, Folders, Oplog and Gitio.
//! Own attempt refs and repeat safety; pending moves and refusal-before-finish ordering.
//! Every case owns its disposable repository/worktrees/remote through the reused support::Fixture.
mod fixture;
mod base_undo;
mod resolve_undo;
mod saves;
mod rebases;
mod refusals;
mod guards;
mod killed;
