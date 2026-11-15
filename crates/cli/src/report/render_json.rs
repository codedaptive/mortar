//! A report as one line of JSON.

use oplog::write_json_string;

use super::{ExitCode, OutValue, Report};
use crate::json::write_json_strings;

/// One JSON object on one line, then "\n". Keys in this order: verb, ok (exit is Done), outcome, exit (the
/// process code), next, lines, owner (a string or null), then the report's fields in their order. A Raw field's
/// values go in unchanged, so the line is JSON only when each of them is.
pub fn render_json(report: &Report) -> String {
    let mut out = String::from("{\"verb\":");
    write_json_string(&mut out, &report.verb);
    out.push_str(&format!(",\"ok\":{},\"outcome\":", report.exit == ExitCode::Done));
    write_json_string(&mut out, &report.outcome);
    out.push_str(&format!(",\"exit\":{},\"next\":", report.exit as i32));
    write_json_string(&mut out, &report.next);
    out.push_str(",\"lines\":");
    write_json_strings(&mut out, &report.lines);
    out.push_str(",\"owner\":");
    match &report.owner {
        Some(owner) => write_json_string(&mut out, owner),
        None => out.push_str("null"),
    }
    for (key, value) in &report.fields {
        out.push(',');
        write_json_string(&mut out, key);
        out.push(':');
        write_value(&mut out, value);
    }
    out.push_str("}\n");
    out
}

fn write_value(out: &mut String, value: &OutValue) {
    match value {
        OutValue::Text(s) => write_json_string(out, s),
        OutValue::Number(n) => out.push_str(&n.to_string()),
        OutValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        OutValue::Null => out.push_str("null"),
        OutValue::Texts(items) => write_json_strings(out, items),
        OutValue::Raw(values) => {
            out.push('[');
            out.push_str(&values.join(","));
            out.push(']');
        }
    }
}
