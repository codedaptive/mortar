//! The numbers step 5 takes for one zone. Data only: measure_zone makes it, run_disk, build_number_lines and
//! kit_number_lines read it, and build_number_lines_tests.rs and kit_number_lines_tests.rs cover that reading.

use std::time::Duration;

/// Each number, or why it was not taken: the zone folder's apparent bytes (.git and target left out), the debug
/// build's wall time, the request folder's apparent bytes (.git and target left out) and again with its build output
/// (.git alone left out), `mr disk`'s object store bytes, and `mr snapshot`'s wall time.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneNumbers {
    pub zone_folder: Result<u64, String>,
    pub build: Result<Duration, String>,
    pub request_folder: Result<u64, String>,
    pub built_request: Result<u64, String>,
    pub store: Result<u64, String>,
    pub snapshot: Result<Duration, String>,
}
