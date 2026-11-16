//! What a child the drive ran ended with. Data only: its methods are in ran_from_output.rs, ran_from_error.rs and
//! ran_seen.rs; ran_seen_tests.rs and ran_from_error_tests.rs cover them.

/// Its exit code or the signal that ended it, and its output as text.
#[derive(Debug, Clone, PartialEq)]
pub struct Ran {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}
