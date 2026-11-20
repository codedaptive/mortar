//! rereview_queries: one git diff from the reviewed commit to the new one, over the whole tree.

use gitio::{BaselineQuery, Oid};

use super::rereview_queries;

const REVIEWED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

#[test]
fn one_diff_from_reviewed_to_new_over_the_whole_tree() {
    assert_eq!(
        rereview_queries(&Oid::parse(REVIEWED).unwrap(), &Oid::parse(NEW).unwrap()),
        vec![BaselineQuery::Diff {
            a: "1111111111111111111111111111111111111111".to_string(),
            b: "2222222222222222222222222222222222222222".to_string(),
            paths: vec![],
        }]
    );
}
