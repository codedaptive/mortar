//! History: the op log's records filtered by ref, key or actor.

mod filter_history;
mod history;
mod history_filter;
mod history_matches;

#[cfg(test)]
mod filter_history_tests;
#[cfg(test)]
mod history_matches_tests;

pub use history::history;
pub use history_filter::HistoryFilter;
pub use history_matches::history_matches;

pub(crate) use filter_history::filter_history;
