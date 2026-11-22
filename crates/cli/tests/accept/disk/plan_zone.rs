//! Plans one of step 5's zones from HEAD's tree.

use super::{plan_build, TreeFile, ZonePlan};

/// `edit_file` is the first file in input order with mode 100644 or 100755 at least one folder below `zone_dir`
/// (which may itself be nested); `request_dir` is its folder; `build` is plan_build at `build_dir`, an Err there
/// leaving the zone planned. Err when the zone holds no such file.
pub fn plan_zone(files: &[TreeFile], zone_dir: &str, build_dir: &str) -> Result<ZonePlan, String> {
    let prefix = format!("{zone_dir}/");
    let edit_file = files
        .iter()
        .filter(|file| file.mode == "100644" || file.mode == "100755")
        .find(|file| file.path.strip_prefix(&prefix).is_some_and(|rest| rest.contains('/')))
        .map(|file| file.path.clone())
        .ok_or_else(|| format!("{zone_dir} holds no regular file in a subfolder"))?;
    Ok(ZonePlan {
        zone_dir: zone_dir.to_string(),
        request_dir: edit_file.rsplit_once('/').map_or_else(String::new, |(dir, _)| dir.to_string()),
        edit_file,
        build: plan_build(files, build_dir),
    })
}
