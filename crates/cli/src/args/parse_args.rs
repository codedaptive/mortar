//! The command line to a verb, its global flags and the rest of its words.

use super::{find_verb, ParsedArgs, UsageError};

/// Reads `argv` (without the program name). Global flags may come anywhere before `--`: `--json`, `--key <v>`
/// or `--key=<v>`, `--actor <v>` or `--actor=<v>`; a missing or empty value, or a flag given twice, is a
/// usage error. The first other word is the verb, and must be in VERBS; every word after it that is not a
/// global, unknown dashes included, goes to `rest` in order. `--` ends the globals: the words after it go to
/// `rest` as they are, and `dash_dash` is where they start in `rest` (a later `--` is one of them). The first
/// error in the line is the one returned, but the whole line is read first so that its `json` says whether
/// `--json` came before `--`.
pub fn parse_args(argv: &[String]) -> Result<ParsedArgs, UsageError> {
    let mut read = Read::default();
    let mut words = argv.iter();
    while let Some(word) = words.next() {
        if word == "--" {
            read.dash_dash = Some(read.rest.len());
            read.rest.extend(words.cloned());
            break;
        }
        let failed = match word.as_str() {
            "--json" if read.json => Some("--json is given twice".to_string()),
            "--json" => {
                read.json = true;
                None
            }
            "--key" => take(&mut read.key, "--key", words.next().map(String::as_str)),
            "--actor" => take(&mut read.actor, "--actor", words.next().map(String::as_str)),
            _ => match (word.strip_prefix("--key="), word.strip_prefix("--actor=")) {
                (Some(value), _) => take(&mut read.key, "--key", Some(value)),
                (_, Some(value)) => take(&mut read.actor, "--actor", Some(value)),
                _ if read.verb.is_none() => {
                    read.verb = Some(word.clone());
                    None
                }
                _ => {
                    read.rest.push(word.clone());
                    None
                }
            },
        };
        if read.error.is_none() {
            read.error = failed;
        }
    }
    finish(read)
}

#[derive(Default)]
struct Read {
    verb: Option<String>,
    rest: Vec<String>,
    dash_dash: Option<usize>,
    json: bool,
    key: Option<String>,
    actor: Option<String>,
    error: Option<String>,
}

/// Stores `value` as `flag`'s value; the reason when it is missing, empty or the flag's second.
fn take(slot: &mut Option<String>, flag: &str, value: Option<&str>) -> Option<String> {
    match value {
        None => Some(format!("{flag} needs a value")),
        Some("") => Some(format!("{flag} needs a non-empty value")),
        Some(_) if slot.is_some() => Some(format!("{flag} is given twice")),
        Some(value) => {
            *slot = Some(value.to_string());
            None
        }
    }
}

fn finish(read: Read) -> Result<ParsedArgs, UsageError> {
    let fail = |reason: String| UsageError { verb: read.verb.clone(), reason, json: read.json };
    if let Some(reason) = read.error.clone() {
        return Err(fail(reason));
    }
    let Some(name) = read.verb.as_deref() else {
        return Err(fail("no verb given".to_string()));
    };
    let Some(verb) = find_verb(name) else {
        return Err(fail(format!("unknown verb {name:?}")));
    };
    Ok(ParsedArgs {
        verb,
        rest: read.rest,
        dash_dash: read.dash_dash,
        json: read.json,
        key: read.key,
        actor: read.actor,
    })
}
