//! The git reads mr status's answer replaces.

use gitio::BaselineQuery;

/// `git status` then `git diff HEAD`, each over the whole folder (no paths).
pub fn status_queries() -> Vec<BaselineQuery> {
    vec![BaselineQuery::Status { paths: vec![] }, BaselineQuery::DiffHead { paths: vec![] }]
}
