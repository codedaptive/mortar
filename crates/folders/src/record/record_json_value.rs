//! A member value of a folder record's JSON object.
//! A bare type; parse_record_json_tests and decode_folder_record_tests exercise it.

/// The only values a record holds: no numbers, booleans or nested objects, so parse_record_json refuses them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RecordJsonValue {
    Null,
    String(String),
    Strings(Vec<String>),
}
