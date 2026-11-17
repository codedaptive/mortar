//! integration_folder_line: the moved folder and its new tip, or that there was none.

use std::path::Path;

use gitio::Oid;

use super::integration_folder_line;

const NEW: &str = "2222222222222222222222222222222222222222";

#[test]
fn a_moved_folder_is_named_with_its_new_tip() {
    let new = Oid::parse(NEW).expect("a valid oid");
    assert_eq!(
        integration_folder_line(Some(Path::new("/r/integration")), &new),
        "moved the integration folder /r/integration to 2222222222222222222222222222222222222222"
    );
}

#[test]
fn no_folder_says_there_was_none_to_move() {
    let new = Oid::parse(NEW).expect("a valid oid");
    assert_eq!(integration_folder_line(None, &new), "no integration folder to move");
}
