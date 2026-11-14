//! take_strings: an array of strings in order, or a refusal naming the key and the item.

use super::{take_strings, ConfigJsonValue as V};

#[test]
fn strings_in_order_with_empties_kept() {
    let value = V::Array(vec![V::String("b".to_string()), V::String(String::new()), V::String("a".to_string())]);
    assert_eq!(take_strings(Some(value), "k"), Ok(vec!["b".to_string(), String::new(), "a".to_string()]));
    assert_eq!(take_strings(Some(V::Array(Vec::new())), "k"), Ok(Vec::new()));
}

#[test]
fn a_non_string_item_is_refused_by_index() {
    let value = V::Array(vec![V::String("a".to_string()), V::Number(1)]);
    assert_eq!(take_strings(Some(value), "root_files"), Err("root_files item 1 is not a string".to_string()));
}

#[test]
fn missing_and_non_arrays_are_refused() {
    assert_eq!(take_strings(None, "root_files"), Err("missing key \"root_files\"".to_string()));
    assert_eq!(take_strings(Some(V::Null), "root_files"), Err("root_files is not an array of strings".to_string()));
}
