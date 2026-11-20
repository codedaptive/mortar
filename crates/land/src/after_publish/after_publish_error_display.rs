//! Display for AfterPublishError: each variant says why and names what it carries; the refusals after_publish makes
//! itself also say that the zone was not moved.

use crate::after_publish::AfterPublishError;
use std::fmt;

impl fmt::Display for AfterPublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AfterPublishError::Git(e) => write!(f, "after-publish git: {e}"),
            AfterPublishError::Squash(e) => write!(f, "after-publish: {e}"),
            AfterPublishError::Join(e) => write!(f, "after-publish replay: {e}"),
            AfterPublishError::Land(e) => write!(f, "after-publish: {e}"),
            AfterPublishError::Unrelated { zone, origin, tip } => write!(
                f,
                "zone {zone} at {tip} shares no history with origin {origin}; the zone was not moved"
            ),
            AfterPublishError::NoSquashOf { zone } => write!(
                f,
                "no squash of zone {zone} on origin carries a Mortar-Squash-Of trailer; the zone was not moved"
            ),
            AfterPublishError::SquashOfNotInZone { zone, squash_of } => write!(
                f,
                "the published squash of zone {zone} was made from {squash_of}, which zone {zone} does not hold; the \
                 zone was not moved"
            ),
        }
    }
}
