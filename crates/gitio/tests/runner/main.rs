//! Real-git integration coverage for WO-gitio-1; each test names the public functions it joins.

#[path = "../support/mod.rs"]
mod support;

mod environment_overrides;
mod exit_codes;
mod hook_environment;
mod pathspec_environment;
mod io_and_cwd;
mod version_floor;
