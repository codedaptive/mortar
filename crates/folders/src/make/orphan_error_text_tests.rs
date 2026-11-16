//! orphan_error_text: every orphan folder named, and each refusal's next step.

use std::path::PathBuf;

use super::orphan_error_text::orphan_error_text;

#[test]
fn orphan_attempt_names_every_folder_and_the_repeat_by_the_harness() {
    let paths = [PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")];
    assert_eq!(
        orphan_error_text(&paths, false),
        "attempt folders /wt/try-1, /wt/try-2 have lost their try branch; nothing changed; next step: keeping or \
         dropping re-makes each from its snapshot first, so have the harness run the same command again"
    );
}

#[test]
fn orphan_without_snapshot_names_every_folder_and_removal_by_hand() {
    let paths = [PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")];
    assert_eq!(
        orphan_error_text(&paths, true),
        "attempt folders /wt/try-1, /wt/try-2 have lost their try branch and have no snapshot, so Mortar holds no \
         copy of them; nothing changed; next step: copy out anything wanted, then remove each folder by hand"
    );
}

#[test]
fn one_folder_is_named_alone() {
    assert_eq!(
        orphan_error_text(&[PathBuf::from("/wt/try-3")], false),
        "attempt folders /wt/try-3 have lost their try branch; nothing changed; next step: keeping or dropping \
         re-makes each from its snapshot first, so have the harness run the same command again"
    );
}
