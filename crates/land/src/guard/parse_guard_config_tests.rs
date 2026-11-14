//! parse_guard_config: text that is not a JSON object is refused before any string is read; every case here is.
//! Cases that read a key or a value run oplog's parse_json_string, so they are Nagatha's RITs (owed, S-101).

use super::parse_guard_config::parse_guard_config;

fn refused(text: &str) -> String {
    match parse_guard_config(text) {
        Ok(refs) => panic!("{text:?} was accepted as {refs:?}"),
        Err(why) => why,
    }
}

#[test]
fn text_that_is_not_an_object_is_refused() {
    for text in ["", "\n", "[]", "\"main\"", "null", "origin_branch=main"] {
        refused(text);
    }
}
