//! A record and its values as one op log line.

use super::{encode_op_record, write_json_string, OpRecord};

/// encode_op_record(r) unchanged when `values` is empty, so no `values` key is written.
/// Otherwise that line with one more member last, before its closing '}':
/// `"values":[{"name":"...","value":"..."},...]`, the pairs in the given order, every string escaped.
pub(crate) fn encode_valued_record(r: &OpRecord, values: &[(String, String)]) -> String {
    let mut out = encode_op_record(r);
    if values.is_empty() {
        return out;
    }
    // encode_op_record's line always ends in the record's closing '}'.
    out.pop();
    out.push_str(",\"values\":[");
    for (i, (name, value)) in values.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"name\":");
        write_json_string(&mut out, name);
        out.push_str(",\"value\":");
        write_json_string(&mut out, value);
        out.push('}');
    }
    out.push_str("]}");
    out
}
