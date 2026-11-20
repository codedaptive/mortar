//! The git read a verb's answer replaces, in words.

/// The words of the git read each counted verb's answer replaces: status, rereview, would-merge and overlap. Every
/// other verb is None. mr log reads Mortar's own op log, not git history, so it replaces no git read.
pub fn verb_baseline(verb: &str) -> Option<&'static str> {
    match verb {
        "status" => Some("git status and git diff HEAD over this folder"),
        "rereview" => Some("git diff <reviewed> <new>"),
        "would-merge" => Some("git log -p <base>..<commit>"),
        "overlap" => Some("git log -p of each request's range"),
        _ => None,
    }
}
