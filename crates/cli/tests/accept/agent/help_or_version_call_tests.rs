use super::help_or_version_call;

fn argv(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

#[test]
fn help_as_the_first_word_asks_for_help() {
    assert!(help_or_version_call(&argv(&["mr", "help"])));
    assert!(help_or_version_call(&argv(&["mr", "help", "land"])));
    assert!(help_or_version_call(&argv(&["mr", "--json", "help", "save"])));
}

#[test]
fn a_help_flag_anywhere_before_the_end_of_the_globals_asks_for_help() {
    assert!(help_or_version_call(&argv(&["mr", "land", "--help"])));
    assert!(help_or_version_call(&argv(&["mr", "folder", "-h"])));
    assert!(help_or_version_call(&argv(&["mr", "--help"])));
    assert!(help_or_version_call(&argv(&["mr", "-h"])));
    assert!(help_or_version_call(&argv(&["mr", "nosuch", "--help"])));
}

#[test]
fn a_help_flag_where_a_keys_value_would_stand_still_counts() {
    assert!(help_or_version_call(&argv(&["mr", "undo", "--key", "--help"])));
}

#[test]
fn version_alone_or_with_one_json_asks_for_the_version() {
    assert!(help_or_version_call(&argv(&["mr", "--version"])));
    assert!(help_or_version_call(&argv(&["mortar", "--version", "--json"])));
    assert!(help_or_version_call(&argv(&["mortar", "--json", "--version"])));
}

#[test]
fn a_verb_call_asks_for_neither() {
    assert!(!help_or_version_call(&argv(&["mr", "status"])));
    assert!(!help_or_version_call(&argv(&["mr", "land"])));
}

#[test]
fn version_with_anything_else_is_not_a_version_call() {
    assert!(!help_or_version_call(&argv(&["mr", "land", "--version"])));
    assert!(!help_or_version_call(&argv(&["mr", "--version", "--version"])));
}

#[test]
fn words_after_the_end_of_the_globals_are_not_read() {
    assert!(!help_or_version_call(&argv(&["mr", "--", "help"])));
    assert!(!help_or_version_call(&argv(&["mr", "bisect", "a", "b", "--", "cmd", "--help"])));
}

#[test]
fn help_as_a_keys_value_or_after_an_unknown_flag_is_not_the_first_word() {
    assert!(!help_or_version_call(&argv(&["mr", "--key", "help", "status"])));
    assert!(!help_or_version_call(&argv(&["mr", "--foo", "help"])));
}

#[test]
fn a_bare_call_asks_for_neither() {
    assert!(!help_or_version_call(&argv(&["mr"])));
}
