//! take_name: a non-empty string, or a refusal naming the key.

use super::{take_name, ConfigJsonValue as V};

#[test]
fn a_string_is_taken() {
    assert_eq!(take_name(Some(V::String("main".to_string())), "k"), Ok("main".to_string()));
}

#[test]
fn empty_missing_and_other_values_are_refused() {
    assert_eq!(take_name(Some(V::String(String::new())), "remote"), Err("remote is empty; it must name something".to_string()));
    assert_eq!(take_name(None, "remote"), Err("missing key \"remote\"".to_string()));
    for value in [V::Null, V::Number(1), V::Array(Vec::new()), V::Object(Vec::new())] {
        assert_eq!(take_name(Some(value), "remote"), Err("remote is not a string".to_string()));
    }
}
