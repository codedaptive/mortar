//! GuardHook: which git hook ran. A bare type; guard_decide_tests exercises it.

/// The hook git ran, its arguments, and for pre-push what the caller read of the guarded remote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardHook {
    /// reference-transaction; `state` is the transaction state git passes (prepared, committed, ...).
    ReferenceTransaction { state: String },
    /// pre-push; `remote` and `url` are the hook's first and second arguments: the remote's name and its URL, or
    /// the URL in both when the push names a URL rather than a remote. `push_urls` is the guarded remote's
    /// (`LandRefs::remote`) push URLs as the caller (the cli's `mortar guard-hook`) reads them with
    /// `git remote get-url --push --all <remote>`: pushurl when set, else url, with insteadOf applied. None when
    /// the caller could not read them; guard_decide then refuses a push under any other name than the guarded one.
    PrePush { remote: String, url: String, push_urls: Option<Vec<String>> },
}
