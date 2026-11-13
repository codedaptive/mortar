//! gitio: see the Mortar mission (Layout).

mod check_exit;
mod classify_update_refs;
mod git;
mod git_command;
mod git_error;
mod git_error_display;
mod git_error_refusal;
mod git_new;
mod git_run;
mod git_update_refs;
mod git_version;
mod lossy_args;
mod oid;
mod oid_as_str;
mod oid_display;
mod oid_parse;
mod output;
mod ref_update;
mod run_opts;
mod settle;
mod spawn_refusal;
mod update_refs_outcome;
mod update_refs_stdin;
mod version;
mod version_parse;
mod version_require_floor;

pub use git::Git;
pub use git_error::GitError;
pub use oid::Oid;
pub use output::Output;
pub use ref_update::RefUpdate;
pub use run_opts::RunOpts;
pub use update_refs_outcome::UpdateRefsOutcome;
pub use version::Version;

#[cfg(test)]
mod check_exit_tests;
#[cfg(test)]
mod classify_update_refs_tests;
#[cfg(test)]
mod git_command_tests;
#[cfg(test)]
mod git_error_display_tests;
#[cfg(test)]
mod git_error_refusal_tests;
#[cfg(test)]
mod git_error_tests;
#[cfg(test)]
mod git_new_tests;
#[cfg(test)]
mod git_run_tests;
#[cfg(test)]
mod git_tests;
#[cfg(test)]
mod git_version_tests;
#[cfg(test)]
mod lossy_args_tests;
#[cfg(test)]
mod oid_as_str_tests;
#[cfg(test)]
mod oid_display_tests;
#[cfg(test)]
mod oid_parse_tests;
#[cfg(test)]
mod oid_tests;
#[cfg(test)]
mod output_tests;
#[cfg(test)]
mod run_opts_tests;
#[cfg(test)]
mod settle_tests;
#[cfg(test)]
mod spawn_refusal_tests;
#[cfg(test)]
mod update_refs_stdin_tests;
#[cfg(test)]
mod version_parse_tests;
#[cfg(test)]
mod version_require_floor_tests;
#[cfg(test)]
mod version_tests;
