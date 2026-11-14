//! One `--zone <zone>=<path>[,<path>...]` of `mortar init`.

use std::collections::BTreeMap;

use crate::config::ZoneConfig;

/// Adds the zone `value` names, with its paths in the order given, to `zones`. Refused: no `=`, an empty zone
/// name, an empty path, or a zone already given.
pub(crate) fn parse_zone_flag(zones: &mut BTreeMap<String, ZoneConfig>, value: &str) -> Result<(), String> {
    let Some((zone, paths)) = value.split_once('=') else {
        return Err(format!("--zone needs <zone>=<path>[,<path>...], not {value:?}"));
    };
    if zone.is_empty() {
        return Err(format!("--zone {value:?} has no zone name"));
    }
    if zones.contains_key(zone) {
        return Err(format!("--zone {zone:?} is given twice"));
    }
    let paths = paths.split(',').map(str::to_string).collect::<Vec<_>>();
    if paths.iter().any(String::is_empty) {
        return Err(format!("--zone {zone:?} has an empty path"));
    }
    zones.insert(zone.to_string(), ZoneConfig { paths });
    Ok(())
}
