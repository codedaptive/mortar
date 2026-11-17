use super::{plan_checkout, CheckoutPlan, TreeFile};

fn file(mode: &str, path: &str, size: Option<u64>) -> TreeFile {
    TreeFile { mode: mode.to_string(), path: path.to_string(), size }
}

#[test]
fn sums_all_sizes_and_selects_the_heaviest_zone_and_first_deep_regular_file() {
    let files = vec![
        file("100644", "root one", Some(8)),
        file("160000", "root-module", None),
        file("100644", "beta/top", Some(7)),
        file("120000", "beta/sub/link", Some(5)),
        file("100755", "beta/sub/deep/run", Some(11)),
        file("100644", "beta/other/later", Some(2)),
        file("100644", "alpha/sub/file", Some(20)),
        file("100644", "root-two", Some(3)),
    ];
    assert_eq!(plan_checkout(&files), Ok(CheckoutPlan {
        bytes: 56,
        zone_dir: "beta".to_string(),
        request_dir: "beta/sub/deep".to_string(),
        edit_file: "beta/sub/deep/run".to_string(),
        root_files: vec!["root one".to_string(), "root-two".to_string()],
    }));
}

#[test]
fn equal_zone_totals_choose_the_lexically_first_name_not_input_order() {
    let files = vec![file("100644", "z/sub/z", Some(10)), file("100644", "a/sub/a", Some(10))];
    assert_eq!(plan_checkout(&files), Ok(CheckoutPlan {
        bytes: 20,
        zone_dir: "a".to_string(),
        request_dir: "a/sub".to_string(),
        edit_file: "a/sub/a".to_string(),
        root_files: vec![],
    }));
}

#[test]
fn no_top_level_folder_is_an_error() {
    assert_eq!(plan_checkout(&[]), Err("HEAD holds no folder at its top level".to_string()));
    assert_eq!(plan_checkout(&[file("100644", "root", Some(1))]), Err("HEAD holds no folder at its top level".to_string()));
}

#[test]
fn a_zone_without_a_deep_regular_file_is_an_error() {
    let files = vec![file("100644", "zone/top", Some(9)), file("120000", "zone/sub/link", Some(2)), file("160000", "zone/sub/module", None)];
    assert_eq!(plan_checkout(&files), Err("zone holds no regular file in a subfolder".to_string()));
}
