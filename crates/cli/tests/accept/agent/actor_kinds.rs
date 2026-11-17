//! Which kinds of op one actor finished, read from a parsed `mr log --json` report.

use crate::json::JsonValue;

/// What a resolve's final op reads as: join's finish_resolve writes it as a "join-rebase" that records the paused
/// join's state among its values under the name "join_state", which no other join-rebase carries.
const RESOLVE: &str = "resolve (join-rebase)";

/// The kind of each record in the report's "records" array whose actor is `actor` and whose phase is "done", in
/// log order, each once; a join-rebase whose "values" hold one named "join_state" reads "resolve (join-rebase)".
/// Empty when the report has no "records" array.
pub fn actor_kinds(report: &JsonValue, actor: &str) -> Vec<String> {
    let Some(JsonValue::Array(records)) = report.field("records") else {
        return Vec::new();
    };
    let mut kinds: Vec<String> = Vec::new();
    for record in records {
        if text(record, "actor") != actor || text(record, "phase") != "done" {
            continue;
        }
        let kind = match (text(record, "kind"), holds_join_state(record)) {
            ("join-rebase", true) => RESOLVE,
            (kind, _) => kind,
        };
        if !kinds.iter().any(|seen| seen == kind) {
            kinds.push(kind.to_string());
        }
    }
    kinds
}

fn holds_join_state(record: &JsonValue) -> bool {
    match record.field("values") {
        Some(JsonValue::Array(values)) => values.iter().any(|value| text(value, "name") == "join_state"),
        _ => false,
    }
}

fn text<'a>(value: &'a JsonValue, name: &str) -> &'a str {
    match value.field(name) {
        Some(JsonValue::Str(text)) => text,
        _ => "",
    }
}
