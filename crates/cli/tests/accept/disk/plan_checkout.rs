//! Plans step 5's zones and folders from HEAD's tree.

use super::{pick_kit, plan_zone, CheckoutPlan, TreeFile};
use std::collections::BTreeMap;

/// `bytes` sums every size (None counts 0). The all-packages zone is the top-level folder whose entries' sizes sum
/// highest, a tie going to the name that sorts first, planned by plan_zone with its build at the copy's top (the
/// copy's own workspace build). `root_files` are the top-level paths with a size, in input order. Err when HEAD has
/// no top-level folder, or that zone holds no regular file in a subfolder.
///
/// The one-kit zone follows the kit rule (pick_kit): of the folders below the top level that directly hold a regular
/// Cargo.toml or Package.swift and also a regular file at least one folder below them, the one whose entries' sizes
/// sum lowest, a tie going to the name that sorts first; plan_zone plans it with its build in the kit's own folder.
/// The build runs in the kit's zone folder, so it relies on `mr folder zone` bringing in the kit's in-repo
/// dependencies; a kit that still cannot build shows as a not-taken build line. No kit, or a kit not
/// planned, is `kit`'s Err and never fails the plan.
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
        .map(|(zone, _)| *zone)
        .ok_or_else(|| "HEAD holds no folder at its top level".to_string())?;
    Ok(CheckoutPlan {
        bytes: files.iter().filter_map(|file| file.size).sum(),
        zone: plan_zone(files, zone_dir, "")?,
        kit: pick_kit(files).and_then(|kit| plan_zone(files, &kit, &kit)),
        root_files: files
            .iter()
            .filter(|file| file.size.is_some() && !file.path.contains('/'))
            .map(|file| file.path.clone())
            .collect(),
    })
}
