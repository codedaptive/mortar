use super::{parse_ls_tree, TreeFile};

#[test]
fn records_preserve_order_modes_paths_and_sizes() {
    let output = "100644 blob abc     12\troot file.txt\0\
100755 blob def      0\tdir/run me\0\
160000 commit 123      -\tmodule\0";
    assert_eq!(parse_ls_tree(output), vec![
        TreeFile { mode: "100644".to_string(), path: "root file.txt".to_string(), size: Some(12) },
        TreeFile { mode: "100755".to_string(), path: "dir/run me".to_string(), size: Some(0) },
        TreeFile { mode: "160000".to_string(), path: "module".to_string(), size: None },
    ]);
}

#[test]
fn malformed_and_empty_records_are_skipped_without_losing_valid_records() {
    let output = "\0not a record\0\
100644 blob abc nope\tbad-size\0\
100644 blob abc 12 missing-tab\0\
100644 blob abc  7\tgood\0\0";
    assert_eq!(parse_ls_tree(output), vec![TreeFile { mode: "100644".to_string(), path: "good".to_string(), size: Some(7) }]);
}

#[test]
fn empty_output_has_no_files() {
    assert_eq!(parse_ls_tree(""), Vec::<TreeFile>::new());
}
