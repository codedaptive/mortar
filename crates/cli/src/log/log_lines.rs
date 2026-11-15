//! The plain lines `mortar log` prints for its records.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove};

use crate::report::is_zero_oid;

/// Per record: "<op_id> <phase> <kind> by <actor> at <at> key <key>: <reason>"; then one line per move,
/// "  <ref> <old|absent> -> <new|deleted>" (deleted: an all-zero new, which deletes the ref); then, for a result
/// other than moved, one line saying what each ref held instead ("  stale: ..." or "  not moved: ..."). No records:
/// the one line "no op in the log matches".
pub fn log_lines(records: &[&OpRecord]) -> Vec<String> {
    if records.is_empty() {
        return vec!["no op in the log matches".to_string()];
    }
    records.iter().flat_map(|record| record_lines(record)).collect()
}

fn record_lines(r: &OpRecord) -> Vec<String> {
    let head = format!(
        "{} {} {} by {} at {} key {}: {}",
        r.op_id,
        phase_name(r.phase),
        r.kind,
        r.actor,
        r.at,
        r.key,
        r.reason
    );
    let result = match &r.result {
        None | Some(OpResult::Moved) => None,
        Some(OpResult::Stale { refname, actual }) => Some(format!("  stale: {}", held(refname, actual.as_ref()))),
        Some(OpResult::NotMoved { refs }) => {
            let held: Vec<String> = refs.iter().map(|(refname, actual)| held(refname, actual.as_ref())).collect();
            Some(format!("  not moved: {}", held.join(", ")))
        }
    };
    std::iter::once(head).chain(r.moves.iter().map(move_line)).chain(result).collect()
}

fn phase_name(phase: OpPhase) -> &'static str {
    match phase {
        OpPhase::Intent => "intent",
        OpPhase::Done => "done",
        OpPhase::Failed => "failed",
    }
}

fn move_line(m: &RefMove) -> String {
    let old = m.old.as_ref().map_or("absent", Oid::as_str);
    let new = if is_zero_oid(&m.new) { "deleted" } else { m.new.as_str() };
    format!("  {} {old} -> {new}", m.refname)
}

/// "<ref> held <oid>" or "<ref> was absent".
fn held(refname: &str, actual: Option<&Oid>) -> String {
    match actual {
        Some(oid) => format!("{refname} held {}", oid.as_str()),
        None => format!("{refname} was absent"),
    }
}
