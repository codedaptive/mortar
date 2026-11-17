//! The report of a command line that cannot be read (exit 4).

use super::{find_verb, UsageError, VERBS};
use crate::report::{ExitCode, Report};

/// Exit Usage, outcome "usage": the reason, then "verbs: " and every verb's name joined by ", ". The next
/// step is the verb's usage line when the verb is known, else "mr <verb>, one of: " and the names.
pub fn usage_report(error: &UsageError) -> Report {
    let names = VERBS.iter().map(|verb| verb.name).collect::<Vec<_>>().join(", ");
    let next = match error.verb.as_deref().and_then(find_verb) {
        Some(verb) => verb.usage.to_string(),
        None => format!("mr <verb>, one of: {names}"),
    };
    Report {
        verb: error.verb.clone().unwrap_or_default(),
        exit: ExitCode::Usage,
        outcome: "usage".to_string(),
        lines: vec![error.reason.clone(), format!("verbs: {names}")],
        owner: None,
        next,
        fields: Vec::new(),
    }
}
