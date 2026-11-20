//! Why text is not a folder record.
//! A bare type; record_error_display_tests exercises its text, decode_folder_record_tests each reason.

/// `reason` names the key, value or byte that is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordError {
    pub reason: String,
}
