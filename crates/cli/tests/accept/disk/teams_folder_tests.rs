use super::teams_folder;
use std::ffi::OsString;
use std::path::PathBuf;

#[test]
fn unset_is_refused_naming_the_variable() {
    assert_eq!(
        teams_folder(None),
        Err("ACCEPT_TEAMS is not set: set it to the folder of codeteam's team manifests".to_string())
    );
}

#[test]
fn empty_is_refused_like_unset() {
    assert_eq!(
        teams_folder(Some(OsString::new())),
        Err("ACCEPT_TEAMS is not set: set it to the folder of codeteam's team manifests".to_string())
    );
}

#[test]
fn a_set_value_is_the_folder() {
    assert_eq!(teams_folder(Some(OsString::from("/tmp/teams"))), Ok(PathBuf::from("/tmp/teams")));
}
