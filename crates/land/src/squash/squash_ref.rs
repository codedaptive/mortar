//! squash_ref: the ref a zone's squash commit lives at, `refs/mortar/squash/<zone>`.

/// `refs/mortar/squash/<zone>`.
pub(crate) fn squash_ref(zone: &str) -> String {
    format!("refs/mortar/squash/{zone}")
}
