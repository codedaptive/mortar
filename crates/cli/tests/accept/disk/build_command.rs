//! The copy's own debug build in a zone folder. Data only: plan_build makes it and time_build runs it, and
//! plan_build_tests.rs covers how it is made.

/// `program args` (always a debug build: the args never hold --release) run in `<zone folder>/<dir>`, `dir` being ""
/// for the zone folder's top.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildCommand {
    pub dir: String,
    pub program: String,
    pub args: Vec<String>,
}
