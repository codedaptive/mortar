//! LandError::State prints "landing: " and LandingStateError's text; a ZIT: it runs that file's Display.

use land::{LandError, LandingStateError};
use std::path::PathBuf;

#[test]
fn state_is_prefixed_with_landing() {
    let inner = LandingStateError::Corrupt { path: PathBuf::from("/r/mortar/landing.json"), reason: "x".to_string() };
    let text = LandError::State(inner).to_string();
    assert!(text.starts_with("landing: landing state /r/mortar/landing.json"), "{text}");
}
