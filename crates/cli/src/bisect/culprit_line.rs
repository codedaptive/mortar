//! What a bisect culprit's squash trailers name, in the cli's words.

use land::Culprit;

/// The change ID and zone the culprit's squash trailers carry, naming each trailer that is missing.
pub fn culprit_line(culprit: &Culprit) -> String {
    match (&culprit.change_id, &culprit.zone) {
        (Some(id), Some(zone)) => format!("its squash is change {id} from zone {zone}"),
        (Some(id), None) => format!("its squash is change {id}; it carries no Mortar-Zone trailer"),
        (None, Some(zone)) => format!("its squash is from zone {zone}; it carries no Mortar-Change-Id trailer"),
        (None, None) => "it carries no squash trailers, so no change ID or zone is named".to_string(),
    }
}
