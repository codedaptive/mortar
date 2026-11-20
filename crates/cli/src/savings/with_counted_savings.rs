//! A report with its savings added, its baseline counted from git reads.

use gitio::{BaselineQuery, GitError};

use super::{verb_baseline, with_savings, Savings};
use crate::report::{ExitCode, Report};

/// The report with Savings { verb_baseline's words, the sum of `count` over `queries` } added by with_savings.
/// Unchanged, and `count` never called, when the report is not Done or its verb has no baseline. Unchanged too when
/// any count fails or the sum overflows: the answer then carries no savings, so no line and no savings.jsonl line,
/// rather than a wrong number.
pub fn with_counted_savings(
    report: Report,
    queries: &[BaselineQuery],
    count: &dyn Fn(&BaselineQuery) -> Result<u64, GitError>,
) -> Report {
    if report.exit != ExitCode::Done {
        return report;
    }
    let Some(words) = verb_baseline(&report.verb) else {
        return report;
    };
    let sum = queries.iter().try_fold(0u64, |sum, query| count(query).ok().and_then(|chars| sum.checked_add(chars)));
    match sum {
        Some(baseline_chars) => with_savings(report, Savings { baseline: words.to_string(), baseline_chars }),
        None => report,
    }
}
