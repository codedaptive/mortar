//! A record, its values and its remote mark as one op log line.
//! Its decisions are proved through append_marked_record by crates/oplog/tests/oplog/marked_lines.rs.

use crate::store::{encode_valued_record, write_json_string, OpRecord};

/// encode_valued_record(r, values) unchanged when `remote` is None, so no `remote` key is written.
/// Otherwise that line with one more member last, before its closing '}': `"remote":"<name>"`, escaped.
pub(crate) fn encode_marked_record(
    r: &OpRecord,
    values: &[(String, String)],
    remote: Option<&str>,
) -> String {
    let mut out = encode_valued_record(r, values);
    let Some(name) = remote else {
        return out;
    };
    // encode_valued_record's line always ends in the record's closing '}'.
    out.pop();
    out.push_str(",\"remote\":");
    write_json_string(&mut out, name);
    out.push('}');
    out
}
