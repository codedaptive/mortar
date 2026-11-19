//! PublishError::State prints "publish: " and LandingStateError's text; a ZIT: it runs that file's Display.

use land::{LandingStateError, PublishError};
use std::path::PathBuf;

#[test]
fn state_is_prefixed_with_publish() {
    let inner = LandingStateError::Corrupt { path: PathBuf::from("/r/mortar/landing.json"), reason: "x".to_string() };
    let text = PublishError::State(inner).to_string();
    assert!(text.starts_with("publish: landing state /r/mortar/landing.json"), "{text}");
}
