//! The words of `mortar guard` after its globals, read into the sub-verb they ask for.

use super::GuardCommand;

/// The first word is the sub-verb, install or status, and nothing may follow it. The Err is the reason: no sub-verb
/// at all, or, naming the word, an unknown sub-verb or a word after the sub-verb.
pub fn parse_guard_args(rest: &[String]) -> Result<GuardCommand, String> {
    let Some((sub, words)) = rest.split_first() else {
        return Err("guard needs a sub-verb: install or status".to_string());
    };
    let command = match sub.as_str() {
        "install" => GuardCommand::Install,
        "status" => GuardCommand::Status,
        other => return Err(format!("unknown guard sub-verb {other:?}")),
    };
    match words.first() {
        Some(extra) => Err(format!("unexpected word {extra:?} in guard {sub}")),
        None => Ok(command),
    }
}
