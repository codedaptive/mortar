//! Zone and request names: what is accepted, each rule's refusal, and that a refusal names both
//! what was checked and the name.

use super::check_name;

fn refused(name: &str) -> String {
    match check_name("zone", name) {
        Err(err) => err.reason,
        Ok(()) => panic!("{name:?} was accepted"),
    }
}

#[test]
fn accepts_letters_digits_and_marks() {
    for name in ["folders", "Zone-9", "a_b.c", "x", "a.b-c_d9"] {
        assert_eq!(check_name("zone", name), Ok(()), "{name:?}");
    }
}

#[test]
fn accepts_exactly_100_bytes() {
    assert_eq!(check_name("zone", &"a".repeat(100)), Ok(()));
}

#[test]
fn refuses_empty() {
    assert_eq!(refused(""), "zone name \"\" is empty");
}

#[test]
fn refuses_101_bytes() {
    let long = "a".repeat(101);
    assert_eq!(refused(&long), format!("zone name {long:?} is over 100 bytes"));
}

#[test]
fn refuses_other_characters() {
    for name in ["a/b", "a b", "a\\b", "a\0b", "a:b", "a~b", "é"] {
        assert!(refused(name).contains("holds a character other than"), "{name:?}");
    }
}

#[test]
fn refuses_leading_dot_or_dash() {
    assert_eq!(refused(".hidden"), "zone name \".hidden\" starts with '.' or '-'");
    assert_eq!(refused("-x"), "zone name \"-x\" starts with '.' or '-'");
}

#[test]
fn refuses_double_dot() {
    assert_eq!(refused("a..b"), "zone name \"a..b\" holds \"..\"");
}

#[test]
fn refuses_lock_or_dot_ending() {
    assert_eq!(refused("a.lock"), "zone name \"a.lock\" ends with \".lock\" or '.'");
    assert_eq!(refused("a."), "zone name \"a.\" ends with \".lock\" or '.'");
}

#[test]
fn accepts_lock_not_at_end() {
    assert_eq!(check_name("zone", "a.locks"), Ok(()));
    assert_eq!(check_name("zone", "lock"), Ok(()));
}

#[test]
fn refusal_names_what_was_checked() {
    let err = check_name("request", "").unwrap_err();
    assert_eq!(err.reason, "request name \"\" is empty");
}
