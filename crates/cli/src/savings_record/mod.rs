//! savings_record: what the edge does with an answer that carries savings once its verb has run: it appends a line
//! to the user's savings.jsonl and, for plain output, reads the user's off switch for the "[mr] tokens saved" line.
//! Both live in the user's XDG folders, read with no repository.

mod append_savings_log;
mod record_savings;
mod rfc3339_utc;
mod savings_line_on;
mod savings_log_line;
mod user_dir;

#[cfg(test)]
mod append_savings_log_tests;
#[cfg(test)]
mod record_savings_tests;
#[cfg(test)]
mod rfc3339_utc_tests;
#[cfg(test)]
mod savings_line_on_tests;
#[cfg(test)]
mod savings_log_line_tests;
#[cfg(test)]
mod user_dir_tests;

use append_savings_log::append_savings_log;
pub(crate) use record_savings::record_savings;
use rfc3339_utc::rfc3339_utc;
use savings_line_on::savings_line_on;
use savings_log_line::savings_log_line;
use user_dir::user_dir;
