//! find_verb: the exact name finds its entry; any other word is None.

use super::find_verb;

#[test]
fn init_is_found() {
    assert_eq!(find_verb("init").map(|verb| verb.name), Some("init"));
}

#[test]
fn other_words_are_none() {
    for word in ["", "Init", "init ", "jump", "--json"] {
        assert_eq!(find_verb(word), None, "{word:?}");
    }
}
