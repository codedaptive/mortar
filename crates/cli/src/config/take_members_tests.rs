//! take_members: one slot per expected key; non-objects, unknown keys and repeats refused by place.

use oplog::JsonValue as V;

use super::take_members;

fn object(members: &[(&str, V)]) -> V {
    V::Object(members.iter().map(|(k, v)| (k.to_string(), v.clone())).collect())
}

#[test]
fn values_land_in_their_key_slots_and_absent_keys_are_none() {
    let value = object(&[("c", V::Number(3)), ("a", V::Null)]);
    assert_eq!(take_members(value, ["a", "b", "c"], "p"), Ok([Some(V::Null), None, Some(V::Number(3))]));
}

#[test]
fn a_non_object_is_refused() {
    assert_eq!(take_members(V::Array(Vec::new()), ["a"], "zones.x"), Err("zones.x is not an object".to_string()));
    assert_eq!(take_members(V::Bool(true), ["a"], "the config"), Err("the config is not an object".to_string()));
}

#[test]
fn an_unknown_key_is_refused() {
    assert_eq!(take_members(object(&[("z", V::Null)]), ["a"], "p"), Err("unknown key \"z\" in p".to_string()));
}

#[test]
fn a_repeated_key_is_refused() {
    let value = object(&[("a", V::Null), ("a", V::Null)]);
    assert_eq!(take_members(value, ["a"], "p"), Err("key \"a\" appears twice in p".to_string()));
}
