//! would_merge_queries: one git log -p from the integration branch's ref to the squash commit, over the whole tree.

use gitio::{BaselineQuery, Oid};

use super::would_merge_queries;

const COMMIT: &str = "3333333333333333333333333333333333333333";

#[test]
fn one_log_patches_from_the_integration_branch_to_the_commit() {
    assert_eq!(
        would_merge_queries("integration", &Oid::parse(COMMIT).unwrap()),
        vec![BaselineQuery::LogPatches {
            range: "refs/heads/integration..3333333333333333333333333333333333333333".to_string(),
            paths: vec![],
        }]
    );
}

#[test]
fn the_branch_name_is_taken_as_given() {
    assert_eq!(
        would_merge_queries("team/main", &Oid::parse(COMMIT).unwrap()),
        vec![BaselineQuery::LogPatches {
            range: "refs/heads/team/main..3333333333333333333333333333333333333333".to_string(),
            paths: vec![],
        }]
    );
}
