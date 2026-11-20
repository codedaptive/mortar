//! What one `mr guard` line asks for: its sub-verb.
//! A bare type; parse_guard_args_tests exercises it.

/// `Install`: put guard.json and both guard hooks in place. `Status`: say whether they are, writing nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardCommand {
    Install,
    Status,
}
