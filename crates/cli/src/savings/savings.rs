//! The savings a verb hands in with its answer.
//! A bare type; savings_line_tests, write_saved_json_tests and with_savings_tests exercise it.

/// `baseline` names, in words, the git read the answer replaces ("git status and git diff HEAD over this folder");
/// `baseline_chars` is that read's output in characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Savings {
    pub baseline: String,
    pub baseline_chars: u64,
}
