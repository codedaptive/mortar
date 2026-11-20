//! The op log store: the record types, their one-line JSON form, and reading and appending the log file
//! `<common>/mortar/oplog.jsonl`.

mod append_op_record;
mod append_valued_record;
mod close_record;
mod decode_op_result;
mod decode_op_values;
mod decode_ref_actual;
mod decode_ref_move;
mod decode_valued_log;
mod encode_op_record;
mod encode_valued_record;
mod intent_record;
mod json_error;
mod json_value;
mod line_error;
mod log_line;
mod marked;
mod move_changes;
mod object_fields;
mod oid_is_zero;
mod op;
mod op_log;
mod op_phase;
mod op_record;
mod op_result;
mod oplog_error;
mod oplog_error_display;
mod oplog_error_source;
mod oplog_path;
mod parse_json;
mod parse_json_string;
mod plain_log;
mod read_op_log;
mod read_ref_value;
mod read_valued_log;
mod ref_move;
mod take_oid;
mod take_optional_oid;
mod take_string;
mod unix_now;
mod valued_log;
mod valued_record;
mod write_json_string;

#[cfg(test)]
mod close_record_tests;
#[cfg(test)]
mod decode_op_result_tests;
#[cfg(test)]
mod decode_op_values_tests;
#[cfg(test)]
mod decode_ref_actual_tests;
#[cfg(test)]
mod decode_ref_move_tests;
#[cfg(test)]
mod decode_valued_log_tests;
#[cfg(test)]
mod encode_op_record_tests;
#[cfg(test)]
mod encode_valued_record_tests;
#[cfg(test)]
mod intent_record_tests;
#[cfg(test)]
mod log_line_tests;
#[cfg(test)]
mod move_changes_tests;
#[cfg(test)]
mod object_fields_tests;
#[cfg(test)]
mod oid_is_zero_tests;
#[cfg(test)]
mod oplog_error_display_tests;
#[cfg(test)]
mod oplog_error_source_tests;
#[cfg(test)]
mod oplog_path_tests;
#[cfg(test)]
mod parse_json_string_tests;
#[cfg(test)]
mod parse_json_tests;
#[cfg(test)]
mod plain_log_tests;
#[cfg(test)]
mod take_oid_tests;
#[cfg(test)]
mod take_optional_oid_tests;
#[cfg(test)]
mod take_string_tests;
#[cfg(test)]
mod write_json_string_tests;

pub use append_op_record::append_op_record;
pub use append_valued_record::append_valued_record;
pub use close_record::close_record;
pub use intent_record::intent_record;
pub use json_error::JsonError;
pub use json_value::JsonValue;
pub use marked::{encode_marked_record, read_marked_log, MarkedLog, MarkedRecord};
pub use op::Op;
pub use op_log::OpLog;
pub use op_phase::OpPhase;
pub use op_record::OpRecord;
pub use op_result::OpResult;
pub use oplog_error::OplogError;
pub use oplog_path::oplog_path;
pub use parse_json::parse_json;
pub use parse_json_string::parse_json_string;
pub use read_op_log::read_op_log;
pub use read_ref_value::read_ref_value;
pub use read_valued_log::read_valued_log;
pub use ref_move::RefMove;
pub use unix_now::unix_now;
pub use valued_log::ValuedLog;
pub use valued_record::ValuedRecord;
pub use write_json_string::write_json_string;

pub(crate) use decode_op_result::decode_op_result;
pub(crate) use decode_op_values::decode_op_values;
pub(crate) use decode_ref_actual::decode_ref_actual;
pub(crate) use decode_ref_move::decode_ref_move;
pub(crate) use decode_valued_log::decode_valued_log;
pub(crate) use encode_op_record::encode_op_record;
pub(crate) use encode_valued_record::encode_valued_record;
pub(crate) use line_error::LineError;
pub(crate) use marked::append_marked_record;
pub(crate) use log_line::log_line;
pub(crate) use move_changes::move_changes;
pub(crate) use object_fields::object_fields;
pub(crate) use oid_is_zero::oid_is_zero;
pub(crate) use plain_log::plain_log;
pub(crate) use take_oid::take_oid;
pub(crate) use take_optional_oid::take_optional_oid;
pub(crate) use take_string::take_string;
