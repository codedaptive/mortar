//! A step's verdict. Data only: block_verdict_tests.rs and drive_passed_tests.rs cover how it is decided and read.

/// PASS, FAIL, or NOT RUN with the reason (the variable to set, or the order that has not filled the step yet).
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Pass,
    Fail,
    NotRun(String),
}
