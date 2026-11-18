//! decide_hooks_path: unset and absolute pass; a relative value, and an empty one, are refused naming the value.

use super::decide_hooks_path::decide_hooks_path;
use super::HooksPathError;

/// The value a Relative refusal names, or None for any other outcome.
fn refused_value(raw: &[u8]) -> Option<String> {
    match decide_hooks_path(Some(raw)) {
        Err(HooksPathError::Relative { value }) => Some(value.to_string_lossy().into_owned()),
        _ => None,
    }
}

#[test]
fn unset_passes() {
    assert!(matches!(decide_hooks_path(None), Ok(())));
}

#[test]
fn absolute_passes() {
    assert!(matches!(decide_hooks_path(Some(b"/abs/hooks\n")), Ok(())));
}

#[test]
fn dot_husky_is_refused_naming_it() {
    assert_eq!(refused_value(b".husky\n").as_deref(), Some(".husky"));
}

#[test]
fn bare_name_is_refused_naming_it() {
    assert_eq!(refused_value(b"hooks\n").as_deref(), Some("hooks"));
}

#[test]
fn dot_slash_is_refused_naming_it() {
    assert_eq!(refused_value(b"./x\n").as_deref(), Some("./x"));
}

#[test]
fn empty_fails_closed() {
    assert_eq!(refused_value(b"\n").as_deref(), Some(""));
}
