//! A Crash as the kill shim reads it. Covered by the shell step (steps/shell.rs), whose before and after kills
//! each depend on the shim reading this value, on every drive.

use super::Crash;

impl Crash {
    /// The value testkit's kill shim reads from MORTAR_TEST_CRASH.
    pub fn as_str(self) -> &'static str {
        match self {
            Crash::Before => "before",
            Crash::After => "after",
        }
    }
}
