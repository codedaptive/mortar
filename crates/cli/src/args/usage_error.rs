//! Why a command line cannot be read.
//! A bare type (S-7); parse_args_tests and crates/cli/tests/usage_report_tests.rs exercise it.

/// `verb` is None when no verb was read, else the word read as the verb (known or not). `reason` names what
/// is wrong. `json` is true when `--json` came before any `--`, so the error renders in the form asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError {
    pub verb: Option<String>,
    pub reason: String,
    pub json: bool,
}
