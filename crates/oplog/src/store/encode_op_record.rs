//! A record as one op log line.

use gitio::Oid;

use super::{write_json_string, OpPhase, OpRecord, OpResult, RefMove};

/// One line of JSON, no trailing newline, keys in the order op_id, key, kind, actor, reason, at, phase,
/// moves, result; `result` is left out when None. Every string is escaped, so any record encodes.
pub(crate) fn encode_op_record(r: &OpRecord) -> String {
    let mut out = String::from("{");
    string_member(&mut out, "op_id", &r.op_id);
    out.push(',');
    string_member(&mut out, "key", &r.key);
    out.push(',');
    string_member(&mut out, "kind", &r.kind);
    out.push(',');
    string_member(&mut out, "actor", &r.actor);
    out.push(',');
    string_member(&mut out, "reason", &r.reason);
    out.push_str(",\"at\":");
    out.push_str(&r.at.to_string());
    out.push(',');
    string_member(&mut out, "phase", phase_name(r.phase));
    out.push_str(",\"moves\":[");
    for (i, m) in r.moves.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        ref_move(&mut out, m);
    }
    out.push(']');
    if let Some(result) = &r.result {
        out.push_str(",\"result\":");
        op_result(&mut out, result);
    }
    out.push('}');
    out
}

fn phase_name(phase: OpPhase) -> &'static str {
    match phase {
        OpPhase::Intent => "intent",
        OpPhase::Done => "done",
        OpPhase::Failed => "failed",
    }
}

fn ref_move(out: &mut String, m: &RefMove) {
    out.push('{');
    string_member(out, "ref", &m.refname);
    out.push_str(",\"old\":");
    optional_oid(out, m.old.as_ref());
    out.push(',');
    string_member(out, "new", m.new.as_str());
    out.push('}');
}

fn op_result(out: &mut String, result: &OpResult) {
    match result {
        OpResult::Moved => out.push_str("{\"outcome\":\"moved\"}"),
        OpResult::NotMoved { refs } => {
            out.push_str("{\"outcome\":\"not-moved\",\"refs\":[");
            for (i, (refname, actual)) in refs.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('{');
                ref_and_actual(out, refname, actual.as_ref());
                out.push('}');
            }
            out.push_str("]}");
        }
        OpResult::Stale { refname, actual } => {
            out.push_str("{\"outcome\":\"stale\",");
            ref_and_actual(out, refname, actual.as_ref());
            out.push('}');
        }
    }
}

/// `"ref":"...","actual":"<hex>"|null`.
fn ref_and_actual(out: &mut String, refname: &str, actual: Option<&Oid>) {
    string_member(out, "ref", refname);
    out.push_str(",\"actual\":");
    optional_oid(out, actual);
}

/// `"name":"value"`, the value escaped.
fn string_member(out: &mut String, name: &str, value: &str) {
    write_json_string(out, name);
    out.push(':');
    write_json_string(out, value);
}

fn optional_oid(out: &mut String, oid: Option<&Oid>) {
    match oid {
        Some(oid) => write_json_string(out, oid.as_str()),
        None => out.push_str("null"),
    }
}
