//! What step 5 measures in the repository copy and edits in the request folder. Data only: plan_checkout makes it,
//! and plan_checkout_tests.rs covers how it is made.

/// `bytes`, a full checkout's apparent bytes; `zone_dir`, the heaviest top-level folder, made the zone; `edit_file`,
/// the first regular file at least one folder below it, and `request_dir`, that file's folder, given to the request
/// folder as its sparse path; `root_files`, the top-level files, each given to `mr init` as a root file.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckoutPlan {
    pub bytes: u64,
    pub zone_dir: String,
    pub request_dir: String,
    pub edit_file: String,
    pub root_files: Vec<String>,
}
