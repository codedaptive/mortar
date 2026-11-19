//! What one run of a verb says, before it is rendered plain or as JSON.
//! A bare type; render_plain_tests and render_json_tests exercise it.

use super::{ExitCode, OutValue};

/// `verb` is "" when no verb was read. `outcome` is one word ("written", "already-done", "refused", "busy",
/// "usage", ...). `lines` say what happened, then where things stand; for a refusal, why. `owner` names who
/// owns the thing refused or busy. `next` is one step, without the "next: " prefix. `fields` are the extra
/// --json members, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verb: String,
    pub exit: ExitCode,
    pub outcome: String,
    pub lines: Vec<String>,
    pub owner: Option<String>,
    pub next: String,
    pub fields: Vec<(String, OutValue)>,
}
