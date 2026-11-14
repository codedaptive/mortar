//! The formatter a team runs before a squash (S-35: the caller sets the command and the timeout).
//! A bare type (S-7); decode_formatter_tests and encode_config_tests exercise it.

/// `command` is the program and its arguments, never empty; `timeout_seconds` bounds one run of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatterConfig {
    pub command: Vec<String>,
    pub timeout_seconds: u64,
}
