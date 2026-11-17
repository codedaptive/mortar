//! The five numbers step 5 takes. Data only: disk_number_lines reads it, and disk_number_lines_tests.rs covers that
//! reading.

use std::time::Duration;

/// Each number, or why it was not taken: a full checkout's apparent bytes, the zone and request folders' apparent
/// bytes, `mr disk`'s object store bytes, and `mr snapshot`'s wall time.
#[derive(Debug, Clone, PartialEq)]
pub struct DiskNumbers {
    pub checkout: Result<u64, String>,
    pub zone_folder: Result<u64, String>,
    pub request_folder: Result<u64, String>,
    pub store: Result<u64, String>,
    pub snapshot: Result<Duration, String>,
}
