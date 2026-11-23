//! `mr would-merge`: its words (the squash and any `--with` squashes), its reports in the cli's own words for the
//! one-squash form and the `--with` form, the git read the one-squash answer replaces, and its runner.

mod parse_would_merge_args;
mod run_would_merge;
mod would_merge_queries;
mod would_merge_report;
mod would_merge_with_report;
mod would_merge_words;

#[cfg(test)]
mod parse_would_merge_args_tests;
#[cfg(test)]
mod would_merge_queries_tests;
#[cfg(test)]
mod would_merge_report_tests;
#[cfg(test)]
mod would_merge_with_report_tests;

pub(crate) use parse_would_merge_args::parse_would_merge_args;
pub(crate) use run_would_merge::run_would_merge;
pub(crate) use would_merge_queries::would_merge_queries;
pub(crate) use would_merge_report::would_merge_report;
pub(crate) use would_merge_with_report::would_merge_with_report;
pub(crate) use would_merge_words::WouldMergeWords;
