//! A request verb's refusal in an attempt folder (S-91): join and resolve act on the request's own folder, so in
//! attempt folder k they run nothing. save is not refused there (it saves under attempt k, S-69, S-83), so the rule
//! is per verb and its caller decides whether to ask.

use folders::{FolderKind, FolderRecord};

use crate::report::{ExitCode, Report};

/// Some for an attempt folder record: exit 1 "refused" naming k and the request, "nothing ran", owned by this
/// request, next "keep or drop the attempts, then mortar join in the request's folder". None for a request or zone
/// folder, and for an attempt record naming no request (decode_folder_record never gives one).
pub fn attempt_refusal(verb: &str, record: &FolderRecord) -> Option<Report> {
    let (FolderKind::Attempt(k), Some(request)) = (record.kind, record.request.as_deref()) else {
        return None;
    };
    Some(Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("this is attempt folder {k} of request {request}: mortar {verb} runs in the request's own folder"),
            "nothing ran".to_string(),
        ],
        owner: Some("this request".to_string()),
        next: "keep or drop the attempts, then mortar join in the request's folder".to_string(),
        fields: Vec::new(),
    })
}
