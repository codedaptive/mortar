//! The filters of `mr log`: --ref from the verb's words, --actor and --key from the global flags.

use oplog::HistoryFilter;

/// `rest` may hold `--ref <r>` or `--ref=<r>`, once; a second --ref, a missing or empty value, or any other word
/// is Err with the reason, which the caller reports as log's usage (exit 2). The filters come Ref, then Actor,
/// then Key; none given is no filter (the whole log).
pub fn log_filters(rest: &[String], key: Option<&str>, actor: Option<&str>) -> Result<Vec<HistoryFilter>, String> {
    let mut refname: Option<String> = None;
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        let value = match word.strip_prefix("--ref=") {
            Some(value) => value,
            None if word == "--ref" => words.next().map(String::as_str).ok_or("--ref needs a value")?,
            None => return Err(format!("unknown word {word:?}")),
        };
        if value.is_empty() {
            return Err("--ref needs a non-empty value".to_string());
        }
        if refname.is_some() {
            return Err("--ref is given twice".to_string());
        }
        refname = Some(value.to_string());
    }
    let filters = [
        refname.map(HistoryFilter::Ref),
        actor.map(|actor| HistoryFilter::Actor(actor.to_string())),
        key.map(|key| HistoryFilter::Key(key.to_string())),
    ];
    Ok(filters.into_iter().flatten().collect())
}
