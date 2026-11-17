//! `mortar bisect`'s words: two commits and --timeout before `--`, the test command after it.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use gitio::Oid;
use land::TestCommand;

use super::BisectArgs;

/// Reads `rest`, whose words from `dash_dash` on came after the line's first `--`. The first failure, in this
/// order, is the reason returned: no `--`; a word before `--` that is a second --timeout, a --timeout with no value
/// before `--`, or any other dashed word; fewer or more than two commit words; a commit word that is not a full
/// commit id (the last passing one first); no --timeout, or a value that is not a whole number of seconds above 0;
/// nothing after `--`. The words after `--` are the command as given: the first is the program, the rest its
/// arguments, global flags and a second `--` included.
pub fn parse_bisect_args(rest: &[String], dash_dash: Option<usize>) -> Result<BisectArgs, String> {
    let Some(k) = dash_dash else {
        return Err("bisect needs `--` before the test command".to_string());
    };
    let (before, after) = rest.split_at(k);
    let (commits, timeout) = split_before(before)?;
    let [last_pass, failed] = commits[..] else {
        return Err(match commits.get(2) {
            Some(word) => format!("unexpected word {word:?} before `--`"),
            None => "bisect needs two commits: the last passing one and the failed one".to_string(),
        });
    };
    let (last_pass, failed) = (commit(last_pass)?, commit(failed)?);
    let Some(timeout) = timeout else {
        return Err("bisect needs --timeout <seconds>".to_string());
    };
    let seconds = match timeout.parse::<u64>() {
        Ok(seconds) if seconds > 0 => seconds,
        _ => return Err(format!("--timeout {timeout:?} is not a whole number of seconds above 0")),
    };
    let Some((program, args)) = after.split_first() else {
        return Err("bisect needs a test command after `--`".to_string());
    };
    Ok(BisectArgs {
        last_pass,
        failed,
        timeout: Duration::from_secs(seconds),
        command: TestCommand { program: PathBuf::from(program), args: args.iter().map(OsString::from).collect() },
    })
}

/// The commit words before `--` in order, and --timeout's value when it is given.
fn split_before(before: &[String]) -> Result<(Vec<&str>, Option<&str>), String> {
    let mut commits = Vec::new();
    let mut timeout = None;
    let mut words = before.iter();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--timeout" if timeout.is_some() => return Err("--timeout is given twice".to_string()),
            "--timeout" => match words.next() {
                Some(value) => timeout = Some(value.as_str()),
                None => return Err("--timeout needs a value".to_string()),
            },
            dashed if dashed.starts_with('-') => return Err(format!("unexpected word {dashed:?} before `--`")),
            word => commits.push(word),
        }
    }
    Ok((commits, timeout))
}

/// `word` as a full commit id.
fn commit(word: &str) -> Result<Oid, String> {
    Oid::parse(word).map_err(|_| format!("{word:?} is not a full commit id (40 or 64 lowercase hex characters)"))
}
