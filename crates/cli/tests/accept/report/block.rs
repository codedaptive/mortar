//! One Done step's block in the report. Data only: format_report_tests.rs covers how it is shown.

use super::{Line, Verdict};

/// The step's number and title, its verdict and its lines in the order they were recorded.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub number: u8,
    pub title: &'static str,
    pub verdict: Verdict,
    pub lines: Vec<Line>,
}
