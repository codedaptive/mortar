//! guard: git hooks and the decision that only Mortar moves integration and origin (SPEC-land 06, WO-land-6).

mod classify_hook;
mod guard_config_path;
mod guard_config_text;
mod guard_decide;
mod guard_error;
mod guard_error_display;
mod guard_error_source;
mod guard_hook;
mod guard_hook_names;
mod guard_hook_script;
mod guard_install;
mod guard_state;
mod guard_status;
mod guard_verdict;
mod hook_state;
mod parse_guard_config;
mod read_guard_config;
mod read_hooks;
mod read_optional;
mod write_replacing;

#[cfg(test)]
mod classify_hook_tests;
#[cfg(test)]
mod guard_config_path_tests;
#[cfg(test)]
mod guard_decide_tests;
#[cfg(test)]
mod guard_error_display_tests;
#[cfg(test)]
mod guard_error_source_tests;
#[cfg(test)]
mod guard_hook_script_tests;
#[cfg(test)]
mod parse_guard_config_tests;

pub use guard_decide::guard_decide;
pub use guard_error::GuardError;
pub use guard_hook::GuardHook;
pub use guard_install::guard_install;
pub use guard_state::GuardState;
pub use guard_status::guard_status;
pub use guard_verdict::GuardVerdict;
pub use hook_state::HookState;
pub use read_guard_config::read_guard_config;
