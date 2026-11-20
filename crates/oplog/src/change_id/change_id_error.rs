//! Every way reading or drawing a change ID can fail.
//! A bare type; change_id_error_display_tests and change_id_error_source_tests exercise each variant.

/// A change ID refusal.
#[derive(Debug)]
pub enum ChangeIdError {
    /// Text that is not 32 chars of k..z; `reason` says what is wrong with it.
    Malformed { text: String, reason: String },
    /// /dev/urandom could not be opened or read.
    Random { source: std::io::Error },
}
