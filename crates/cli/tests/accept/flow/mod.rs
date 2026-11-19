//! Step 1, the flow: run_flow, its check groups in check order, the state they hand on, and the pure
//! decisions they make from what Mortar and git said.

mod conflict_commit;
mod files_in;
mod flow_checks;
mod flow_conflict;
mod flow_land;
mod flow_plain_git;
mod flow_publish;
mod flow_rebase;
mod flow_replay;
mod flow_requests;
mod flow_rereview;
mod flow_squash;
mod flow_state;
mod not_run;
mod one_parent_each;
mod requests_missing;
mod rereview_changes;
mod rereview_files;
mod rereview_only_edit;
mod run_flow;
mod sparse_strays;
mod trailers_missing;
mod worktrees_missing;

#[cfg(test)]
mod not_run_tests;
#[cfg(test)]
mod one_parent_each_tests;
#[cfg(test)]
mod requests_missing_tests;
#[cfg(test)]
mod rereview_changes_tests;
#[cfg(test)]
mod rereview_files_tests;
#[cfg(test)]
mod rereview_only_edit_tests;
#[cfg(test)]
mod sparse_strays_tests;
#[cfg(test)]
mod trailers_missing_tests;
#[cfg(test)]
mod worktrees_missing_tests;

pub use conflict_commit::conflict_commit;
pub use files_in::files_in;
pub use flow_checks::FLOW_CHECKS;
pub use flow_conflict::flow_conflict;
pub use flow_land::flow_land;
pub use flow_plain_git::flow_plain_git;
pub use flow_publish::flow_publish;
pub use flow_rebase::flow_rebase;
pub use flow_replay::flow_replay;
pub use flow_requests::flow_requests;
pub use flow_rereview::flow_rereview;
pub use flow_squash::flow_squash;
pub use flow_state::FlowState;
pub use not_run::not_run;
pub use one_parent_each::one_parent_each;
pub use requests_missing::requests_missing;
pub use rereview_changes::rereview_changes;
pub use rereview_files::rereview_files;
pub use rereview_only_edit::rereview_only_edit;
pub use run_flow::run_flow;
pub use sparse_strays::sparse_strays;
pub use trailers_missing::trailers_missing;
pub use worktrees_missing::worktrees_missing;
