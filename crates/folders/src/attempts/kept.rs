//! What keeping one attempt does, as keep_plan plans it.
//! A bare type; keep_op_tests and tests/folders/attempt_plans.rs exercise it.

use std::path::PathBuf;

use gitio::Oid;

/// `tip`: the kept attempt's tip, where the request branch moves. `remove`: the attempt folders to remove, the
/// kept attempt's last. `delete`: every attempt branch (full name) with the tip it was read at, to delete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub tip: Oid,
    pub remove: Vec<PathBuf>,
    pub delete: Vec<(String, Oid)>,
}
