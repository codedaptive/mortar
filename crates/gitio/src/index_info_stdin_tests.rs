use crate::index_info_stdin::index_info_stdin;
use crate::Oid;
use std::path::PathBuf;

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

fn entry(mode: &str, hex: &str) -> Option<(String, Oid)> {
    Some((mode.to_string(), Oid::parse(hex).unwrap()))
}

#[test]
fn one_entry_is_mode_id_tab_path_nul() {
    let bytes = index_info_stdin(&paths(&["a/f"]), &[entry("100644", "5626abf0f72e58d7a153368ba57db4c673c0e171")], 40);
    assert_eq!(bytes, b"100644 5626abf0f72e58d7a153368ba57db4c673c0e171\ta/f\x00".to_vec());
}

#[test]
fn a_removal_is_mode_zero_with_40_zeros() {
    assert_eq!(
        index_info_stdin(&paths(&["gone"]), &[None], 40),
        b"0 0000000000000000000000000000000000000000\tgone\x00".to_vec()
    );
}

#[test]
fn a_removal_is_mode_zero_with_64_zeros() {
    assert_eq!(
        index_info_stdin(&paths(&["gone"]), &[None], 64),
        b"0 0000000000000000000000000000000000000000000000000000000000000000\tgone\x00".to_vec()
    );
}

#[test]
fn mixed_items_keep_their_order() {
    let bytes = index_info_stdin(
        &paths(&["z", "gone", "link"]),
        &[
            entry("100755", "1111111111111111111111111111111111111111"),
            None,
            entry("120000", "4d1ae35ba2c8ec712fa2a379db44ad639ca277bd"),
        ],
        40,
    );
    assert_eq!(
        bytes,
        b"100755 1111111111111111111111111111111111111111\tz\x00\
          0 0000000000000000000000000000000000000000\tgone\x00\
          120000 4d1ae35ba2c8ec712fa2a379db44ad639ca277bd\tlink\x00"
            .to_vec()
    );
}

#[test]
fn a_path_holding_a_newline_and_a_tab_goes_in_raw() {
    let bytes = index_info_stdin(&paths(&["n\nl\tt"]), &[entry("100644", "1111111111111111111111111111111111111111")], 40);
    assert_eq!(bytes, b"100644 1111111111111111111111111111111111111111\tn\nl\tt\x00".to_vec());
}

#[test]
fn no_paths_is_no_input() {
    assert_eq!(index_info_stdin(&[], &[], 40), Vec::<u8>::new());
}
