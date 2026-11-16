//! last_trailer_value: the value of a key that is present; None for an absent key and an empty list; keys match in
//! any ASCII case; a repeated key gives its last value; a key that only starts the same is not a match.

use super::last_trailer_value;

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect()
}

#[test]
fn a_present_key_gives_its_value() {
    let trailers = pairs(&[("Mortar-Zone", "land"), ("Mortar-Squash-Of", "4444444444444444444444444444444444444444")]);
    assert_eq!(last_trailer_value(&trailers, "Mortar-Zone"), Some("land"));
}

#[test]
fn an_absent_key_is_none() {
    let trailers = pairs(&[("Mortar-Zone", "land")]);
    assert_eq!(last_trailer_value(&trailers, "Mortar-Squash-Of"), None);
}

#[test]
fn an_empty_list_is_none() {
    assert_eq!(last_trailer_value(&[], "Mortar-Zone"), None);
}

#[test]
fn a_lowercase_key_is_found() {
    let trailers = pairs(&[("mortar-zone", "land")]);
    assert_eq!(last_trailer_value(&trailers, "Mortar-Zone"), Some("land"));
}

#[test]
fn a_mixed_case_key_is_found() {
    let trailers = pairs(&[("MORTAR-zone", "land")]);
    assert_eq!(last_trailer_value(&trailers, "Mortar-Zone"), Some("land"));
}

#[test]
fn a_repeated_key_gives_the_last_value() {
    let trailers = pairs(&[("Mortar-Zone", "join"), ("Signed-off-by", "a"), ("Mortar-Zone", "land")]);
    assert_eq!(last_trailer_value(&trailers, "Mortar-Zone"), Some("land"));
}

#[test]
fn a_key_that_only_starts_the_same_is_not_a_match() {
    let trailers = pairs(&[("Mortar-Zone-Old", "join"), ("Mortar-Zon", "x")]);
    assert_eq!(last_trailer_value(&trailers, "Mortar-Zone"), None);
}
