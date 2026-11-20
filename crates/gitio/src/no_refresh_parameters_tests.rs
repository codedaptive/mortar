use crate::no_refresh_parameters::no_refresh_parameters;
use std::ffi::{OsStr, OsString};

#[test]
fn no_base_gives_just_the_entry() {
    assert_eq!(no_refresh_parameters(None), OsString::from("'diff.autoRefreshIndex'='false'"));
}

#[test]
fn an_empty_base_gives_just_the_entry() {
    assert_eq!(
        no_refresh_parameters(Some(OsStr::new(""))),
        OsString::from("'diff.autoRefreshIndex'='false'")
    );
}

#[test]
fn an_existing_entry_is_kept_first() {
    assert_eq!(
        no_refresh_parameters(Some(OsStr::new("'core.x'='y'"))),
        OsString::from("'core.x'='y' 'diff.autoRefreshIndex'='false'")
    );
}

#[test]
fn an_existing_refresh_setting_is_kept_and_ours_comes_last() {
    assert_eq!(
        no_refresh_parameters(Some(OsStr::new("'diff.autoRefreshIndex'='true'"))),
        OsString::from("'diff.autoRefreshIndex'='true' 'diff.autoRefreshIndex'='false'")
    );
}
