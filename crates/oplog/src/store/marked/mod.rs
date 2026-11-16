//! Op log lines with a remote mark: a record, its values and the optional `remote` key written last.

mod append_marked_record;
mod decode_marked_line;
mod decode_marked_log;
mod encode_marked_record;
mod marked_log;
mod marked_record;
mod read_marked_log;
mod unmarked_log;

#[cfg(test)]
mod unmarked_log_tests;

pub use append_marked_record::append_marked_record;
pub use decode_marked_log::decode_marked_log;
pub use encode_marked_record::encode_marked_record;
pub use marked_log::MarkedLog;
pub use marked_record::MarkedRecord;
pub use read_marked_log::read_marked_log;

pub(crate) use decode_marked_line::decode_marked_line;
pub(crate) use unmarked_log::unmarked_log;
