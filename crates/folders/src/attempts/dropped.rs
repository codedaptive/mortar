//! What dropping a request's attempts does, as drop_plan plans it.
//! A bare type; drop_op_tests and tests/folders/attempt_plans.rs exercise it.

use std::path::PathBuf;

use gitio::Oid;

/// `remove`: the attempt folders to remove: those of attempts with a branch in k order, then those whose branch is
/// gone in k order. `delete`: every attempt branch (full name) with the tip it was read at, to delete, in k order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropped {
    pub remove: Vec<PathBuf>,
    pub delete: Vec<(String, Oid)>,
}
