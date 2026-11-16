//! The "records" lines of `mortar log`.

use oplog::{encode_op_record, OpRecord};

/// With `json`, each record's line as oplog's encode_op_record encodes it (the record alone, without values or a
/// remote mark), in the order given; without it, no lines, since the plain report never prints its fields (S-114).
pub fn encoded_records(records: &[&OpRecord], json: bool) -> Vec<String> {
    if !json {
        return Vec::new();
    }
    records.iter().map(|record| encode_op_record(record)).collect()
}
