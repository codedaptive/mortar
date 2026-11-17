//! Step 4, the agent test (04-agent.md): run_agent, its wrappers and agent run, its checks in order, and the pure
//! readings of the wrappers' logs and the op log.

mod actor_kinds;
mod agent_brief;
mod agent_checks;
mod agent_conflict_check;
mod agent_files;
mod agent_files_new;
mod agent_git_check;
mod agent_log_check;
mod agent_not_run;
mod agent_session;
mod agent_verbs_check;
mod agent_verbs_outside_six;
mod agent_zone_check;
mod conflict_seen;
mod git_call_is_read;
mod log_call;
mod log_call_from_agent;
mod log_call_shown;
mod mr_verb;
mod non_read_git_calls;
mod parse_log_line;
mod run_agent;
mod write_agent_bin;

#[cfg(test)]
mod actor_kinds_tests;
#[cfg(test)]
mod agent_files_new_tests;
#[cfg(test)]
mod agent_not_run_tests;
#[cfg(test)]
mod agent_verbs_outside_six_tests;
#[cfg(test)]
mod conflict_seen_tests;
#[cfg(test)]
mod git_call_is_read_tests;
#[cfg(test)]
mod log_call_from_agent_tests;
#[cfg(test)]
mod log_call_shown_tests;
#[cfg(test)]
mod mr_verb_tests;
#[cfg(test)]
mod non_read_git_calls_tests;
#[cfg(test)]
mod parse_log_line_tests;

pub use actor_kinds::actor_kinds;
pub use agent_brief::AGENT_BRIEF;
pub use agent_checks::AGENT_CHECKS;
pub use agent_conflict_check::agent_conflict_check;
pub use agent_files::AgentFiles;
pub use agent_git_check::agent_git_check;
pub use agent_log_check::agent_log_check;
pub use agent_not_run::agent_not_run;
pub use agent_session::agent_session;
pub use agent_verbs_check::agent_verbs_check;
pub use agent_verbs_outside_six::agent_verbs_outside_six;
pub use agent_zone_check::agent_zone_check;
pub use conflict_seen::conflict_seen;
pub use git_call_is_read::git_call_is_read;
pub use log_call::LogCall;
pub use mr_verb::mr_verb;
pub use non_read_git_calls::non_read_git_calls;
pub use parse_log_line::parse_log_line;
pub use run_agent::run_agent;
pub use write_agent_bin::write_agent_bin;
