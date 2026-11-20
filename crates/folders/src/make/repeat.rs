//! Whether a folder already at the path is the one asked for.
//! A bare type; repeat_decision_tests exercises both values.

/// `Same`: the folder asked for is already there, so making it again is a repeat. `Different`: another folder
/// is there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    Same,
    Different,
}
