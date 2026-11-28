//! keeps_save_trailers: whether a message's trailers still carry exactly the save's Mortar trailers, so run_save_hooks
//! knows whether to add them again after the commit-msg hook. keeps_save_trailers_tests.rs checks it.

/// True when the pairs of `pairs` whose key is one of `wanted`'s keys are exactly `wanted`, in order: keys compared
/// ASCII case-insensitively, as git and oplog match trailer keys, and values exactly; pairs with other keys are
/// ignored.
///
/// A commit-msg hook may drop or change the Mortar trailers, so run_save_hooks adds them again with
/// Git::with_trailers when this is false. It is not done unconditionally because git's default trailer rule
/// (addIfDifferentNeighbor) compares a new trailer only with the last one, so adding them to a message that still
/// carries them would repeat all three on every save.
pub(crate) fn keeps_save_trailers(pairs: &[(String, String)], wanted: &[(String, String)]) -> bool {
    let mut kept = pairs.iter().filter(|(key, _)| wanted.iter().any(|(want, _)| want.eq_ignore_ascii_case(key)));
    let same = wanted.iter().all(|(want_key, want_value)| {
        kept.next().is_some_and(|(key, value)| key.eq_ignore_ascii_case(want_key) && value == want_value)
    });
    same && kept.next().is_none()
}
