//! mr_path_warning: no line when the first `mr` on PATH is this build; else the line naming it, or saying there is
//! none, and naming `mortar` at this binary's path.

use std::path::Path;

use super::mr_path_warning;

#[test]
fn the_first_mr_being_this_build_gives_no_warning() {
    assert_eq!(
        mr_path_warning(Some(Path::new("/b/target/debug/mr")), Path::new("/b/target/debug/mortar"), true),
        None
    );
}

#[test]
fn another_first_mr_is_named_and_mortar_is_the_way_in() {
    assert_eq!(
        mr_path_warning(Some(Path::new("/usr/local/bin/mr")), Path::new("/b/target/debug/mortar"), false),
        Some(
            "the first mr on PATH is /usr/local/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn no_mr_on_path_says_so_and_mortar_is_the_way_in() {
    assert_eq!(
        mr_path_warning(None, Path::new("/b/target/debug/mortar"), false),
        Some("no mr is on PATH: run Mortar as mortar (/b/target/debug/mortar)".to_string())
    );
}

#[test]
fn no_mr_on_path_warns_whatever_same_says() {
    assert_eq!(
        mr_path_warning(None, Path::new("/opt/mortar/bin/mortar"), true),
        Some("no mr is on PATH: run Mortar as mortar (/opt/mortar/bin/mortar)".to_string())
    );
}
