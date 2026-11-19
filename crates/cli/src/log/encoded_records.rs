//! The "records" lines of `mr log`: each record's own op log line.

use oplog::{encode_marked_record, MarkedRecord};

/// With `json`, each record's line as oplog's encode_marked_record encodes it (the record, its values and its
/// remote mark, as the log holds it), in the order given; without it, no lines, since the plain report never prints
/// its fields.
pub fn encoded_records(records: &[&MarkedRecord], json: bool) -> Vec<String> {
    if !json {
        return Vec::new();
    }
    records.iter().map(|m| encode_marked_record(&m.record, &m.values, m.remote.as_deref())).collect()
}
