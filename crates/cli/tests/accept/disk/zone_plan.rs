//! What step 5 makes, edits and builds for one zone. Data only: plan_zone makes it, and plan_zone_tests.rs covers
//! how it is made.

use super::BuildCommand;

/// `zone_dir`, the folder made into the zone; `edit_file`, the first regular file at least one folder below it, and
/// `request_dir`, that file's folder, given to the request folder as its sparse path; `build`, the debug build run in
/// the zone folder, or why there is none.
#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlan {
    pub zone_dir: String,
    pub request_dir: String,
    pub edit_file: String,
    pub build: Result<BuildCommand, String>,
}
