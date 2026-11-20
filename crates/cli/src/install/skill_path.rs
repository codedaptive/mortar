//! skill_path: Where a harness's Mortar skill file goes (spec 07 item 5's table).

use std::path::{Path, PathBuf};

use super::{HarnessSpec, SkillLocation};

/// (home for Global, project for Local)/`harness.skills`/mortar/SKILL.md.
pub fn skill_path(harness: &HarnessSpec, location: SkillLocation, home: &Path, project: &Path) -> PathBuf {
    let base = match location {
        SkillLocation::Global => home,
        SkillLocation::Local => project,
    };
    base.join(harness.skills).join("mortar").join("SKILL.md")
}
