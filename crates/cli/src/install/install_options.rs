//! What `mr install` was asked to do.
//! A bare type (S-7); parse_install_args_tests exercises it.

use super::{HarnessSpec, SkillLocation};

/// `no_place`: --no-place. `short_name`: --short-name, link `mr` even behind a foreign `mr` on PATH. `targets`:
/// --target's harnesses in HARNESSES order, None when not given. `yes`: -y, wire every detected harness without
/// asking. `location`: --location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOptions {
    pub no_place: bool,
    pub short_name: bool,
    pub targets: Option<Vec<&'static HarnessSpec>>,
    pub yes: bool,
    pub location: SkillLocation,
}
