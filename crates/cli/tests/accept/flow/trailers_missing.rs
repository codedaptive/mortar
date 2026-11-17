//! Which of Mortar's three save trailers a commit lacks (check 2).

/// `trailers` is git's `%(trailers:only,unfold)` of the commit: one `Key: value` line each. Names, in this order,
/// what is wrong: `Mortar-Change-Id` when it is missing or empty, `Mortar-Request: <request>` and
/// `Mortar-Zone: <zone>` when that trailer is missing or holds another value. Keys match ASCII case-insensitively, as
/// git matches them, and the last of a repeated key counts. Empty when all three are there.
pub fn trailers_missing(trailers: &str, request: &str, zone: &str) -> Vec<String> {
    let value = |key: &str| {
        trailers
            .lines()
            .filter_map(|line| line.split_once(':'))
            .filter(|(name, _)| name.trim().eq_ignore_ascii_case(key))
            .map(|(_, value)| value.trim())
            .last()
    };
    let mut missing = Vec::new();
    if value("Mortar-Change-Id").is_none_or(str::is_empty) {
        missing.push("Mortar-Change-Id".to_string());
    }
    if value("Mortar-Request") != Some(request) {
        missing.push(format!("Mortar-Request: {request}"));
    }
    if value("Mortar-Zone") != Some(zone) {
        missing.push(format!("Mortar-Zone: {zone}"));
    }
    missing
}
