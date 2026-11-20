//! decode_zones: an object of `{"paths": [...]}` entries by zone name.

use std::collections::BTreeMap;

use oplog::JsonValue as V;

use super::{decode_zones, ZoneConfig};

fn zone(paths: &[&str]) -> V {
    V::Object(vec![("paths".to_string(), V::Array(paths.iter().map(|p| V::String(p.to_string())).collect()))])
}

#[test]
fn zones_by_name() {
    let value = V::Object(vec![("web".to_string(), zone(&["w", "x"])), ("api".to_string(), zone(&[]))]);
    let mut want = BTreeMap::new();
    want.insert("api".to_string(), ZoneConfig { paths: Vec::new() });
    want.insert("web".to_string(), ZoneConfig { paths: vec!["w".to_string(), "x".to_string()] });
    assert_eq!(decode_zones(Some(value)), Ok(want));
    assert_eq!(decode_zones(Some(V::Object(Vec::new()))), Ok(BTreeMap::new()));
}

#[test]
fn a_zone_given_twice_is_refused() {
    let value = V::Object(vec![("a".to_string(), zone(&["1"])), ("a".to_string(), zone(&["2"]))]);
    assert_eq!(decode_zones(Some(value)), Err("zone \"a\" appears twice in zones".to_string()));
}

#[test]
fn missing_and_non_object_zones_are_refused() {
    assert_eq!(decode_zones(None), Err("missing key \"zones\"".to_string()));
    assert_eq!(decode_zones(Some(V::Null)), Err("zones is not an object".to_string()));
    assert_eq!(decode_zones(Some(V::Bool(true))), Err("zones is not an object".to_string()));
}
