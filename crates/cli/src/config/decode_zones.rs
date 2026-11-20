//! The config's `zones` member to zone entries.

use std::collections::BTreeMap;

use oplog::JsonValue;

use super::{take_members, take_strings, ZoneConfig};

/// `zones` must be an object whose every member is `{"paths": [strings]}` with exactly that key. A zone
/// name given twice is refused: the map would keep only one of them silently.
pub(crate) fn decode_zones(slot: Option<JsonValue>) -> Result<BTreeMap<String, ZoneConfig>, String> {
    let members = match slot {
        Some(JsonValue::Object(members)) => members,
        Some(_) => return Err("zones is not an object".to_string()),
        None => return Err("missing key \"zones\"".to_string()),
    };
    let mut zones = BTreeMap::new();
    for (name, value) in members {
        if zones.contains_key(&name) {
            return Err(format!("zone {name:?} appears twice in zones"));
        }
        let place = format!("zones.{name}");
        let [paths] = take_members(value, ["paths"], &place)?;
        let paths = take_strings(paths, &format!("{place}.paths"))?;
        zones.insert(name, ZoneConfig { paths });
    }
    Ok(zones)
}
