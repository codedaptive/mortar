//! What install decided about the short-name link `~/.local/bin/mr` (spec 07 item 3).
//! A bare type (S-7); plan_link_tests exercises it.

use super::{InstallItem, InstallStep};

/// `step`: the link to make, if any. `item`: its report line. `present`: once install is done, Mortar's link stands
/// at `~/.local/bin/mr`. `next`: a next step this decision forces, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPlan {
    pub step: Option<InstallStep>,
    pub item: InstallItem,
    pub present: bool,
    pub next: Option<String>,
}
