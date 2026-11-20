//! scan_help_words: which words of a command line are globals, help flags or other words.
//! Unit test: scan_help_words is pure.

use super::{scan_help_words, HelpWords};

fn scan(words: &[&str]) -> (bool, bool, Vec<String>) {
    let argv = words.iter().map(|word| (*word).to_string()).collect::<Vec<_>>();
    let HelpWords { json, flagged, words } = scan_help_words(&argv);
    (json, flagged, words.into_iter().map(str::to_string).collect())
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| (*word).to_string()).collect()
}

#[test]
fn an_empty_line_has_nothing() {
    assert_eq!(scan(&[]), (false, false, words(&[])));
}

#[test]
fn json_and_the_help_flags_are_noted_and_dropped() {
    assert_eq!(scan(&["--json", "save", "-h"]), (true, true, words(&["save"])));
    assert_eq!(scan(&["--help", "save", "x"]), (false, true, words(&["save", "x"])));
}

#[test]
fn a_second_json_is_no_error() {
    assert_eq!(scan(&["--json", "--json", "help"]), (true, false, words(&["help"])));
}

#[test]
fn key_and_actor_drop_their_value() {
    assert_eq!(scan(&["--key", "k", "--actor", "a", "save"]), (false, false, words(&["save"])));
}

#[test]
fn a_help_flag_in_a_value_position_still_counts() {
    assert_eq!(scan(&["--key", "-h", "save"]), (false, true, words(&["save"])));
    assert_eq!(scan(&["undo", "--key", "--help"]), (false, true, words(&["undo"])));
    assert_eq!(scan(&["--actor", "-h"]), (false, true, words(&[])));
}

#[test]
fn a_dash_dash_taken_as_a_value_does_not_end_the_globals() {
    assert_eq!(scan(&["--key", "--", "help"]), (false, false, words(&["help"])));
}

#[test]
fn a_value_joined_by_equals_is_one_word() {
    assert_eq!(scan(&["--key=k", "--actor=a", "save", "-h"]), (false, true, words(&["save"])));
}

#[test]
fn a_flag_without_its_value_ends_the_read() {
    assert_eq!(scan(&["-h", "--actor"]), (false, true, words(&[])));
}

#[test]
fn the_dash_dash_ends_the_globals() {
    assert_eq!(scan(&["--", "help", "--json", "-h"]), (false, false, words(&[])));
    assert_eq!(scan(&["bisect", "a", "--", "cmd", "--help"]), (false, false, words(&["bisect", "a"])));
}
