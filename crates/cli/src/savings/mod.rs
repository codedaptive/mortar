//! savings: the tokens an answer saves against the git read it replaces: the Savings value a verb hands in, its
//! plain "[mr] tokens saved" line and its --json object, the answer's own character count, adding and removing the
//! value on a report, adding it with its baseline counted from git reads, each verb's baseline in words, and the
//! lines `mr help` adds about it.

mod answer_chars;
mod report_savings;
mod savings;
mod savings_help_lines;
mod savings_line;
mod tokens_of;
mod verb_baseline;
mod with_counted_savings;
mod with_savings;
mod with_thousands;
mod without_savings;
mod write_saved_json;

#[cfg(test)]
mod answer_chars_tests;
#[cfg(test)]
mod report_savings_tests;
#[cfg(test)]
mod savings_help_lines_tests;
#[cfg(test)]
mod savings_line_tests;
#[cfg(test)]
mod savings_tests;
#[cfg(test)]
mod tokens_of_tests;
#[cfg(test)]
mod verb_baseline_tests;
#[cfg(test)]
mod with_counted_savings_tests;
#[cfg(test)]
mod with_savings_tests;
#[cfg(test)]
mod with_thousands_tests;
#[cfg(test)]
mod without_savings_tests;
#[cfg(test)]
mod write_saved_json_tests;

pub(crate) use answer_chars::answer_chars;
pub(crate) use report_savings::report_savings;
pub(crate) use savings::Savings;
pub(crate) use savings_help_lines::savings_help_lines;
pub(crate) use savings_line::savings_line;
pub(crate) use tokens_of::tokens_of;
pub(crate) use verb_baseline::verb_baseline;
pub(crate) use with_counted_savings::with_counted_savings;
pub(crate) use with_savings::with_savings;
use with_thousands::with_thousands;
pub(crate) use without_savings::without_savings;
pub(crate) use write_saved_json::write_saved_json;
