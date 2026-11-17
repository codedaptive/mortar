//! parse_guard_args: install and status alone, and each word it refuses.

use super::{parse_guard_args, GuardCommand};

fn parse(words: &[&str]) -> Result<GuardCommand, String> {
    parse_guard_args(&words.iter().map(ToString::to_string).collect::<Vec<_>>())
}

#[test]
fn install_and_status_take_no_words() {
    assert_eq!(parse(&["install"]), Ok(GuardCommand::Install));
    assert_eq!(parse(&["status"]), Ok(GuardCommand::Status));
}

#[test]
fn no_sub_verb_is_refused() {
    assert_eq!(parse(&[]), Err("guard needs a sub-verb: install or status".to_string()));
}

#[test]
fn an_unknown_sub_verb_is_refused_by_name() {
    assert_eq!(parse(&["frobnicate"]), Err("unknown guard sub-verb \"frobnicate\"".to_string()));
    assert_eq!(parse(&[""]), Err("unknown guard sub-verb \"\"".to_string()));
    assert_eq!(parse(&["Install"]), Err("unknown guard sub-verb \"Install\"".to_string()));
}

#[test]
fn a_word_after_the_sub_verb_is_refused_by_name() {
    assert_eq!(parse(&["install", "now"]), Err("unexpected word \"now\" in guard install".to_string()));
    assert_eq!(parse(&["status", "--all", "x"]), Err("unexpected word \"--all\" in guard status".to_string()));
}
