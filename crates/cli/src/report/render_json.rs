//! A report as one line of JSON.

use oplog::write_json_string;

use super::{ExitCode, OutValue, Report};
use crate::json::write_json_strings;
use crate::savings::{answer_chars, write_saved_json};

/// One JSON object on one line, then "\n". Keys in this order: verb, ok (exit is Done), outcome, exit (the
/// process code), next, lines, owner (a string or null), then the report's fields in their order. A Raw field's
/// values go in unchanged, so the line is JSON only when each of them is. An Object field is `{"name":value,...}`, its
/// members in their order, each value written as a field's is. A Saved field is its savings object, whose
/// outputChars is the characters of the report's plain body without the savings line (answer_chars).
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
        write_value(&mut out, value, report);
    }
    out.push_str("}\n");
    out
}

fn write_value(out: &mut String, value: &OutValue, report: &Report) {
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
        OutValue::Object(members) => {
            out.push('{');
            for (index, (name, member)) in members.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_json_string(out, name);
                out.push(':');
                write_value(out, member, report);
            }
            out.push('}');
        }
        OutValue::Saved(savings) => write_saved_json(out, savings, answer_chars(report)),
    }
}
