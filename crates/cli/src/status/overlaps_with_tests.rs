//! overlaps_with: the pairs naming the request, first or second, as the other name and the paths, in pair order.

use std::path::PathBuf;

use ::join::OverlapPair;

use super::overlaps_with;

fn pair(first: &str, second: &str, paths: &[&str]) -> OverlapPair {
    OverlapPair {
        first: first.to_string(),
        second: second.to_string(),
        paths: paths.iter().map(PathBuf::from).collect(),
    }
}

#[test]
fn pairs_naming_the_request_either_way_give_the_other_name_in_pair_order() {
    let pairs = [
        pair("fix-0", "fix-1", &["a.rs"]),
        pair("fix-2", "fix-3", &["b.rs"]),
        pair("fix-1", "fix-4", &["c.rs", "d.rs"]),
    ];
    assert_eq!(
        overlaps_with("fix-1", &pairs),
        vec![
            ("fix-0".to_string(), vec![PathBuf::from("a.rs")]),
            ("fix-4".to_string(), vec![PathBuf::from("c.rs"), PathBuf::from("d.rs")]),
        ]
    );
}

#[test]
fn no_pair_naming_the_request_is_no_overlap() {
    assert_eq!(overlaps_with("fix-1", &[pair("fix-2", "fix-3", &["b.rs"])]), Vec::new());
    assert_eq!(overlaps_with("fix-1", &[]), Vec::new());
}

#[test]
fn a_name_that_only_starts_like_the_request_is_not_it() {
    assert_eq!(overlaps_with("fix-1", &[pair("fix-10", "fix-2", &["b.rs"])]), Vec::new());
}
