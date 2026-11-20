//! `mr sweep`'s words: --idle-days <n> at most once, --ref <full ref name> any number of times.

use super::SweepArgs;

/// Reads `rest`; any `--` on the line is refused. The first failure in line order is the reason returned: a second
/// --idle-days; --idle-days with no value or one that is not a whole number of days (0 allowed); --ref with no
/// value; any other word. --ref values are kept in order, duplicates as given.
pub fn parse_sweep_args(rest: &[String], dash_dash: Option<usize>) -> Result<SweepArgs, String> {
    if dash_dash.is_some() {
        return Err("sweep takes no `--`".to_string());
    }
    let mut parsed = SweepArgs { idle_days: None, refs: Vec::new() };
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--idle-days" if parsed.idle_days.is_some() => return Err("--idle-days is given twice".to_string()),
            "--idle-days" => {
                let Some(value) = words.next() else {
                    return Err("--idle-days needs a whole number of days".to_string());
                };
                parsed.idle_days = Some(whole_days(value)?);
            }
            "--ref" => match words.next() {
                Some(refname) => parsed.refs.push(refname.clone()),
                None => return Err("--ref needs a full ref name".to_string()),
            },
            other => return Err(format!("unexpected word {other:?}")),
        }
    }
    Ok(parsed)
}

/// `value` as a whole number of days: ASCII digits only (u64's parse alone would take a leading `+`), within u64.
fn whole_days(value: &str) -> Result<u64, String> {
    let refused = || format!("--idle-days needs a whole number of days, not {value:?}");
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(refused());
    }
    value.parse::<u64>().map_err(|_| refused())
}
