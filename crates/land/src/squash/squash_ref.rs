//! squash_ref: the ref a zone's squash commit lives at (S-57).

/// `refs/mortar/squash/<zone>`.
pub(crate) fn squash_ref(zone: &str) -> String {
    format!("refs/mortar/squash/{zone}")
}
