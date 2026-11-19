use super::checks_git_floor;
use crate::args::VERBS;

const EXEMPT: [&str; 4] = ["help", "install", "uninstall", "guard-hook"];

#[test]
fn exempt_verbs_do_not_check_the_git_floor() {
    for verb in EXEMPT {
        assert!(!checks_git_floor(verb), "{verb}");
    }
}

#[test]
fn every_other_known_verb_checks_the_git_floor() {
    for verb in VERBS {
        assert_eq!(checks_git_floor(verb.name), !EXEMPT.contains(&verb.name), "{}", verb.name);
    }
}

#[test]
fn every_exempt_verb_is_in_the_verb_table() {
    for exempt in EXEMPT {
        assert!(VERBS.iter().any(|verb| verb.name == exempt), "{exempt}");
    }
}
