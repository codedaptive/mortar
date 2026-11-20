//! The part of a report that a join answer gives.
//! A bare type; answer_part makes it and answer_part_tests exercises it.

use crate::report::{ExitCode, OutValue};

/// The answer's `lines` and --json `fields`, in order, and the report's `exit`, `outcome` and `next`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerPart {
    pub lines: Vec<String>,
    pub fields: Vec<(String, OutValue)>,
    pub exit: ExitCode,
    pub outcome: &'static str,
    pub next: &'static str,
}
