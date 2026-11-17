//! One call the `mr`/`mortar` wrappers or the git shim logged. Data only: parse_log_line makes it, and
//! parse_log_line_tests.rs, log_call_from_agent_tests.rs and log_call_shown_tests.rs cover how it is made and read.

/// The process's pid, its parent's pid, its parent's command base name ("drive" in mr.log for the drive's own
/// conflicting change and every `mortar` run under it), and its argv, argv[0] being the program's base name.
#[derive(Debug, Clone, PartialEq)]
pub struct LogCall {
    pub pid: u32,
    pub ppid: u32,
    pub parent: String,
    pub args: Vec<String>,
}
