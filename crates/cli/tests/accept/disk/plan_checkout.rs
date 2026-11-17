//! Plans step 5's folders from HEAD's tree.

use super::{CheckoutPlan, TreeFile};
use std::collections::BTreeMap;

/// `bytes` sums every size (None counts 0). `zone_dir` is the top-level folder whose entries' sizes sum highest, a
/// tie going to the name that sorts first. `edit_file` is the first file in input order with mode 100644 or 100755
/// at least one folder below `zone_dir`; `request_dir` is its folder. `root_files` are the top-level paths with a
/// size, in input order. Err when HEAD has no top-level folder, or the zone has no such file.
pub fn plan_checkout(files: &[TreeFile]) -> Result<CheckoutPlan, String> {
    let mut zones: BTreeMap<&str, u64> = BTreeMap::new();
    for file in files {
        if let Some((zone, _)) = file.path.split_once('/') {
            *zones.entry(zone).or_default() += file.size.unwrap_or(0);
        }
    }
    // BTreeMap iterates in name order and max_by_key keeps the last maximum, so iterate reversed: the first name wins.
    let zone_dir = zones
        .iter()
        .rev()
        .max_by_key(|(_, bytes)| **bytes)
        .map(|(zone, _)| zone.to_string())
        .ok_or_else(|| "HEAD holds no folder at its top level".to_string())?;
    let prefix = format!("{zone_dir}/");
    let edit_file = files
        .iter()
        .filter(|file| file.mode == "100644" || file.mode == "100755")
        .find(|file| file.path.strip_prefix(&prefix).is_some_and(|rest| rest.contains('/')))
        .map(|file| file.path.clone())
        .ok_or_else(|| format!("{zone_dir} holds no regular file in a subfolder"))?;
    Ok(CheckoutPlan {
        bytes: files.iter().filter_map(|file| file.size).sum(),
        zone_dir,
        request_dir: edit_file.rsplit_once('/').map_or_else(String::new, |(dir, _)| dir.to_string()),
        edit_file,
        root_files: files
            .iter()
            .filter(|file| file.size.is_some() && !file.path.contains('/'))
            .map(|file| file.path.clone())
            .collect(),
    })
}
