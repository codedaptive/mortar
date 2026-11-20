//! JoinedAnswer: what `mr disk` can say about whether a folder's request has joined its zone.
//! A bare type; holds_line_tests, holds_json_tests and disk_report_tests build it.

/// Joined or NotJoined: the request or attempt folder's joined answer (status's rule). ZoneFolder: a zone folder,
/// which has no request and so no joined answer. Unknown: the joined read failed; the text is why, in the cli's
/// words (joined_unknown_reason).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinedAnswer {
    Joined,
    NotJoined,
    ZoneFolder,
    Unknown(String),
}
