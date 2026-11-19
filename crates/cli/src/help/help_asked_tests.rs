//! help_asked: which command lines ask for help, for which topic, and whether as JSON.
//! Unit test: scan_help_words, the help groups and find_help_entry are pure and used directly; the scan's
//! own cases are in scan_help_words_tests.

use super::{help_asked, HelpTopic};

/// The answer for `words` as plain words: None, or (json, "program" | "verb <group> <name>" | "unknown <word>" |
/// "too-many"), so each expectation is a literal.
fn asked(words: &[&str]) -> Option<(bool, String)> {
    let argv = words.iter().map(|word| (*word).to_string()).collect::<Vec<_>>();
    help_asked(&argv).map(|request| {
        let topic = match request.topic {
            HelpTopic::Program => "program".to_string(),
            HelpTopic::Verb(group, entry) => format!("verb {} {}", group.key, entry.name),
            HelpTopic::Unknown(word) => format!("unknown {word}"),
            HelpTopic::TooMany => "too-many".to_string(),
        };
        (request.json, topic)
    })
}

fn yes(json: bool, topic: &str) -> Option<(bool, String)> {
    Some((json, topic.to_string()))
}

#[test]
fn help_alone_is_the_program_help() {
    assert_eq!(asked(&["help"]), yes(false, "program"));
}

#[test]
fn the_help_flags_alone_are_the_program_help() {
    assert_eq!(asked(&["--help"]), yes(false, "program"));
    assert_eq!(asked(&["-h"]), yes(false, "program"));
    assert_eq!(asked(&["--json", "-h"]), yes(true, "program"));
}

#[test]
fn every_form_of_one_verb_asks_for_its_help() {
    for words in [
        &["help", "save"][..],
        &["save", "--help"],
        &["save", "-h"],
        &["--help", "save"],
        &["save", "x", "--actor", "a", "-h"],
    ] {
        assert_eq!(asked(words), yes(false, "verb agent save"), "{words:?}");
    }
}

#[test]
fn version_has_a_help() {
    assert_eq!(asked(&["help", "--version"]), yes(false, "verb setup --version"));
    assert_eq!(asked(&["--version", "--help"]), yes(false, "verb setup --version"));
}

#[test]
fn help_has_a_help() {
    assert_eq!(asked(&["help", "help"]), yes(false, "verb setup help"));
}

#[test]
fn help_of_an_unknown_verb_is_unknown() {
    assert_eq!(asked(&["help", "nosuch"]), yes(false, "unknown nosuch"));
}

#[test]
fn help_with_two_verbs_is_too_many() {
    assert_eq!(asked(&["help", "save", "join"]), yes(false, "too-many"));
}

#[test]
fn lines_that_do_not_ask_for_help_are_none() {
    for words in [
        &["nosuch", "--help"][..],
        &["save"],
        &["bisect", "a", "b", "--timeout", "5", "--", "cmd", "--help"],
        &["--", "help"],
        &[],
    ] {
        assert_eq!(asked(words), None, "{words:?}");
    }
}

#[test]
fn help_wins_over_a_global_flag_error() {
    assert_eq!(asked(&["--json", "--json", "help"]), yes(true, "program"));
}
