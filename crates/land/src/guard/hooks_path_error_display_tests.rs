//! HooksPathError texts. The Git arm prints gitio's own text, so its case runs another crate's Display: it is in
//! tests/land_rit/check_hooks_path.rs, not here.

use std::path::PathBuf;

use super::HooksPathError;

#[test]
fn relative_names_the_value_and_why() {
    let err = HooksPathError::Relative { value: PathBuf::from(".husky") };
    assert_eq!(
        err.to_string(),
        "core.hooksPath is \".husky\", not an absolute path: git resolves a relative hooks path in each folder, so one \
         install cannot guard them all"
    );
}

#[test]
fn relative_empty_shows_empty_quotes() {
    let err = HooksPathError::Relative { value: PathBuf::from("") };
    assert_eq!(
        err.to_string(),
        "core.hooksPath is \"\", not an absolute path: git resolves a relative hooks path in each folder, so one \
         install cannot guard them all"
    );
}
