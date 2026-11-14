use crate::tree_with_paths_requests::tree_with_paths_requests;
use crate::{Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

const BASE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const INDEX: &str = "/tmp/mortar-index-7-99-0";

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn requests(stdin: &[u8]) -> [(Vec<OsString>, RunOpts); 3] {
    tree_with_paths_requests(&Oid::parse(BASE).unwrap(), Path::new(INDEX), stdin.to_vec())
}

#[test]
fn first_run_reads_the_base_tree() {
    assert_eq!(
        requests(b"")[0].0,
        argv(&["read-tree", "--end-of-options", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"])
    );
}

#[test]
fn second_run_updates_the_index_from_stdin_with_index_info_last() {
    let [_, update, _] = requests(b"0 0000000000000000000000000000000000000000\tgone\x00");
    assert_eq!(update.0, argv(&["update-index", "-z", "--index-info"]));
    assert_eq!(update.1.stdin, Some(b"0 0000000000000000000000000000000000000000\tgone\x00".to_vec()));
}

#[test]
fn empty_input_is_still_fed_to_update_index() {
    assert_eq!(requests(b"")[1].1.stdin, Some(Vec::new()));
}

#[test]
fn third_run_writes_the_tree() {
    assert_eq!(requests(b"")[2].0, argv(&["write-tree"]));
}

#[test]
fn read_tree_and_write_tree_have_no_stdin() {
    let [read_tree, _, write_tree] = requests(b"x");
    assert_eq!(read_tree.1.stdin, None);
    assert_eq!(write_tree.1.stdin, None);
}

#[test]
fn every_run_uses_the_temp_index_only_and_writes() {
    for (_, opts) in requests(b"x") {
        assert_eq!(
            opts.env,
            vec![(OsString::from("GIT_INDEX_FILE"), OsString::from("/tmp/mortar-index-7-99-0"))]
        );
        assert!(!opts.read_only);
        assert!(opts.answer_codes.is_empty());
        assert_eq!(opts.cwd, None);
    }
}
