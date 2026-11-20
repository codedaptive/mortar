//! The git read mr rereview's answer replaces.

use gitio::{BaselineQuery, Oid};

/// `git diff <reviewed> <new>` over the whole tree (no paths).
pub fn rereview_queries(reviewed: &Oid, new: &Oid) -> Vec<BaselineQuery> {
    vec![BaselineQuery::Diff { a: reviewed.as_str().to_string(), b: new.as_str().to_string(), paths: vec![] }]
}
