//! What install will do, decided in full before anything is written.
//! A bare type; plan_install_tests and install_report_tests exercise it.

use super::{InstallItem, InstallStep};

/// `steps`: the writes, in order. `items`: the report's lines. `next`: the report's `next:` step. `command`: the
/// planned name Mortar goes by once install is done, "mr" or "mortar", as skill_command chose it; apply_failure names
/// "mr" only once the link was applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    pub steps: Vec<InstallStep>,
    pub items: Vec<InstallItem>,
    pub next: String,
    pub command: &'static str,
}
