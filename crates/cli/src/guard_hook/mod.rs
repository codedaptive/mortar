//! `mortar guard-hook <name>` (SPEC-land 06): the hook's words read as a land::GuardHook, the guard's verdict as a
//! report, the guard's verdict with the open intent read only when the verdict can depend on it, and the runner the
//! guard's git hooks call.

mod decide_with_intent;
mod guard_hook_from_args;
mod guard_hook_report;
mod run_guard_hook;

#[cfg(test)]
mod decide_with_intent_tests;
#[cfg(test)]
mod guard_hook_from_args_tests;
#[cfg(test)]
mod guard_hook_report_tests;

pub(crate) use decide_with_intent::decide_with_intent;
pub(crate) use guard_hook_from_args::guard_hook_from_args;
pub(crate) use guard_hook_report::guard_hook_report;
pub(crate) use run_guard_hook::run_guard_hook;
