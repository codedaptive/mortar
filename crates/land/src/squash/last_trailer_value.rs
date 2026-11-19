//! last_trailer_value: one trailer's value among a commit's trailer pairs. last_trailer_value_tests covers it.

/// The value of the last pair whose key equals `key` ASCII case-insensitively, as git matches trailer keys; None
/// when no key does. The last pair counts because Mortar writes its trailers last, the rule oplog's
/// pick_mortar_trailers follows.
pub(crate) fn last_trailer_value<'a>(trailers: &'a [(String, String)], key: &str) -> Option<&'a str> {
    trailers.iter().rev().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
}
