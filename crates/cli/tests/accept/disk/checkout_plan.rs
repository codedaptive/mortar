//! What step 5 measures in the repository copy, makes into zones and edits in their request folders. Data only:
//! plan_checkout makes it, and plan_checkout_tests.rs covers how it is made.

use super::ZonePlan;

/// `bytes`, a full checkout's apparent bytes; `zone`, the all-packages zone (the heaviest top-level folder, built at
/// the copy's top); `kit`, the one-kit zone, or why there is none; `root_files`, the top-level files, each given to
/// `mr init` as a root file.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckoutPlan {
    pub bytes: u64,
    pub zone: ZonePlan,
    pub kit: Result<ZonePlan, String>,
    pub root_files: Vec<String>,
}
