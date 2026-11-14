//! One op record with its values and its remote mark.
//! A bare type; crates/oplog/tests/oplog/marked_lines.rs exercises it.

use crate::store::OpRecord;

/// `values` as in ValuedRecord. `remote` is the remote's name on an intent opened by open_remote_intent and
/// on its closing record; None on every other record, whose line has no `remote` key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkedRecord {
    pub record: OpRecord,
    pub values: Vec<(String, String)>,
    pub remote: Option<String>,
}
