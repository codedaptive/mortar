//! `mortar guard-hook <name>` (SPEC-land 06): the hook's words read as a land::GuardHook, the guard's verdict as a
//! report, and the runner the guard's git hooks call.

mod guard_hook_from_args;
mod guard_hook_report;
mod run_guard_hook;

#[cfg(test)]
mod guard_hook_from_args_tests;
#[cfg(test)]
mod guard_hook_report_tests;

pub use guard_hook_from_args::guard_hook_from_args;
pub use guard_hook_report::guard_hook_report;
pub use run_guard_hook::run_guard_hook;
