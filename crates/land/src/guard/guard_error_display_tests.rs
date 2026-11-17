//! Guard failure texts. The Git arm prints gitio's own text, so its case runs another crate's Display: a RIT, in
//! tests/land_rit/owed_displays.rs, not here. Io embeds std's text, so only its path prefix is checked.

use std::io;
use std::path::PathBuf;

use super::GuardError;

#[test]
fn io_text_starts_with_the_path() {
    let err = GuardError::Io { path: PathBuf::from("/r/.git/hooks/pre-push"), error: io::Error::other("disk") };
    assert!(err.to_string().starts_with("/r/.git/hooks/pre-push: "), "{err}");
}

#[test]
fn foreign_hook_names_the_file_and_what_to_do() {
    let err = GuardError::ForeignHook { path: PathBuf::from("/r/.git/hooks/pre-push") };
    assert_eq!(
        err.to_string(),
        "/r/.git/hooks/pre-push is a hook Mortar did not write, and Mortar never overwrites one: move it aside or fold \
         it in by hand, then install the guard again"
    );
}

#[test]
fn bad_config_names_the_file_and_why() {
    let err = GuardError::BadConfig {
        path: PathBuf::from("/r/.git/mortar/guard.json"),
        why: "key \"remote\" appears twice".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "/r/.git/mortar/guard.json is not a Mortar guard config: key \"remote\" appears twice"
    );
}
