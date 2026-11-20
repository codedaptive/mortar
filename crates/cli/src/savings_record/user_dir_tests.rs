//! user_dir: an absolute XDG folder wins; else the home folder's relative path; else None.

use std::ffi::OsStr;
use std::path::PathBuf;

use super::user_dir;

fn os(text: &str) -> Option<&OsStr> {
    Some(OsStr::new(text))
}

#[test]
fn an_absolute_xdg_wins() {
    assert_eq!(user_dir(os("/x/state"), os("/home/u"), ".local/state"), Some(PathBuf::from("/x/state/mortar")));
    assert_eq!(user_dir(os("/x/config"), None, ".config"), Some(PathBuf::from("/x/config/mortar")));
}

#[test]
fn a_relative_or_empty_xdg_falls_back_to_home() {
    assert_eq!(user_dir(os("rel/state"), os("/home/u"), ".local/state"), Some(PathBuf::from("/home/u/.local/state/mortar")));
    assert_eq!(user_dir(os(""), os("/home/u"), ".config"), Some(PathBuf::from("/home/u/.config/mortar")));
    assert_eq!(user_dir(None, os("/home/u"), ".config"), Some(PathBuf::from("/home/u/.config/mortar")));
}

#[test]
fn neither_is_none() {
    assert_eq!(user_dir(None, None, ".config"), None);
    assert_eq!(user_dir(os(""), os(""), ".config"), None);
    assert_eq!(user_dir(os("rel"), None, ".local/state"), None);
}
