//! FetchRefOutcome: what fetching one ref did. A declaration only; classify_fetch_ref_tests exercises it.

/// The ref was fetched, or the remote has no ref by the source name and nothing was written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchRefOutcome {
    /// The destination ref now holds the remote's source ref, forced even when that is not a fast-forward.
    Fetched,
    /// The remote has no ref by the source name; the destination ref is untouched.
    MissingSource,
}
