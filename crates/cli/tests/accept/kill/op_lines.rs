//! Reads the op log records out of a parsed `mr log --json` report.

use super::OpLine;
use crate::json::JsonValue;

/// Each object in the report's "records" array, in order, as an OpLine; an item that is not an object is left out,
/// and a missing or non-string member reads as "". Empty when the report has no "records" array.
pub fn op_lines(report: &JsonValue) -> Vec<OpLine> {
    let Some(JsonValue::Array(records)) = report.field("records") else {
        return Vec::new();
    };
    records.iter().filter(|record| matches!(record, JsonValue::Object(_))).map(op_line).collect()
}

fn op_line(record: &JsonValue) -> OpLine {
    let moves = match record.field("moves") {
        Some(JsonValue::Array(moves)) => moves.iter().map(|one| (text(one, "ref"), text(one, "new"))).collect(),
        _ => Vec::new(),
    };
    OpLine {
        op_id: text(record, "op_id"),
        kind: text(record, "kind"),
        phase: text(record, "phase"),
        result: record.field("result").map(|result| text(result, "outcome")).unwrap_or_default(),
        reason: text(record, "reason"),
        moves,
    }
}

fn text(value: &JsonValue, name: &str) -> String {
    match value.field(name) {
        Some(JsonValue::Str(text)) => text.clone(),
        _ => String::new(),
    }
}
