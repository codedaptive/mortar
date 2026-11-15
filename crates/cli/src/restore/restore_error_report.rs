//! The report of an op log refusal during `mortar restore`.

use oplog::OplogError;

use crate::report::{failure_report, ExitCode, Report};

/// `text` is `error` as the op log words it (its Display, passed in by the caller), which already says what to do.
/// UnknownOp, ValueUnknown, BadMove and Protected are exit 1 "refused" with `text` as the one line and a next
/// step that runs now (S-112): UnknownOp "mortar log"; ValueUnknown "mortar log --ref <ref>"; BadMove and Protected
/// in words, Protected owned by its protector. Anything else is the op log or git failing: failure_report.
pub fn restore_error_report(error: &OplogError, text: &str) -> Report {
    let (next, owner) = match error {
        OplogError::UnknownOp { .. } => ("mortar log".to_string(), None),
        OplogError::ValueUnknown { refname, .. } => (format!("mortar log --ref {refname}"), None),
        OplogError::BadMove { refname, .. } => {
            (format!("name {refname:?} by its full, valid ref name, then repeat"), None)
        }
        OplogError::Protected { refname, by, .. } => {
            (format!("restore without {refname}, or ask {by}, which protects it"), Some(by.clone()))
        }
        _ => return failure_report("restore", text),
    };
    Report {
        verb: "restore".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![text.to_string()],
        owner,
        next,
        fields: Vec::new(),
    }
}
