//! placement_skip: --no-place, then an unknown path, then a package-manager prefix; None otherwise.

use std::path::{Path, PathBuf};

use super::placement_skip;

fn prefixes() -> Vec<PathBuf> {
    vec![PathBuf::from("/opt/homebrew"), PathBuf::from("/usr/local")]
}

#[test]
fn a_binary_outside_every_prefix_is_placed() {
    assert_eq!(placement_skip(Some(Path::new("/home/ana/build/mortar")), &prefixes(), false), None);
}

#[test]
fn no_place_is_the_reason_before_any_other() {
    let under = Some(Path::new("/opt/homebrew/bin/mortar"));
    assert_eq!(placement_skip(under, &prefixes(), true).as_deref(), Some("--no-place"));
    assert_eq!(placement_skip(None, &prefixes(), true).as_deref(), Some("--no-place"));
}

#[test]
fn a_binary_under_a_prefix_names_the_binary_and_the_prefix() {
    assert_eq!(
        placement_skip(Some(Path::new("/opt/homebrew/Cellar/mortar/0.1/bin/mortar")), &prefixes(), false).as_deref(),
        Some("/opt/homebrew/Cellar/mortar/0.1/bin/mortar is under /opt/homebrew, a package manager's prefix")
    );
    assert_eq!(
        placement_skip(Some(Path::new("/usr/local/bin/mortar")), &prefixes(), false).as_deref(),
        Some("/usr/local/bin/mortar is under /usr/local, a package manager's prefix")
    );
}

#[test]
fn a_prefix_matches_whole_components_only() {
    assert_eq!(placement_skip(Some(Path::new("/usr/localx/bin/mortar")), &prefixes(), false), None);
    assert_eq!(placement_skip(Some(Path::new("/opt/homebrewed/mortar")), &prefixes(), false), None);
}

#[test]
fn an_unknown_path_is_skipped() {
    assert_eq!(placement_skip(None, &prefixes(), false).as_deref(), Some("this binary's path is unknown"));
}
