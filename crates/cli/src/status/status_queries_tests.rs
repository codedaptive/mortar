//! status_queries: git status and git diff HEAD over the whole folder, in that order.

use gitio::BaselineQuery;

use super::status_queries;

#[test]
fn status_then_diff_head_over_the_whole_folder() {
    assert_eq!(
        status_queries(),
        vec![BaselineQuery::Status { paths: vec![] }, BaselineQuery::DiffHead { paths: vec![] }]
    );
}
