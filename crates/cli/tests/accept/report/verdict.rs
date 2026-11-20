//! A step's verdict. Data only: block_verdict_tests.rs and drive_passed_tests.rs cover how it is decided and read.

/// PASS, FAIL, or NOT RUN with the reason (the variable to set, or the acceptance item whose step is not written yet).
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Pass,
    Fail,
    NotRun(String),
}
