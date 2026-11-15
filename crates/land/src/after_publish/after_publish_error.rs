//! AfterPublishError: why after_publish stopped. A declaration only (S-7); after_publish_error_display_tests
//! exercises the NoSquashOf text, after_publish_error_source_tests its source; the Unrelated and SquashOfNotInZone
//! texts print Oids and are Nagatha's RITs (owed, S-101).

use crate::landing::LandError;
use crate::squash::SquashError;
use gitio::{GitError, Oid};
use join::JoinError;

/// An after_publish refusal; the zones before the one it names were already moved. Debug only: SquashError and
/// LandError are neither Clone nor Eq.
#[derive(Debug)]
pub enum AfterPublishError {
    /// A git run failed, or a Mortar-Squash-Of value is not an object id.
    Git(GitError),
    /// The origin branch or a zone branch could not be read: absent, or git failed.
    Squash(SquashError),
    /// Replaying a zone onto origin failed before it could answer clean or conflicted.
    Join(JoinError),
    /// The op moving a zone branch was refused or failed.
    Land(LandError),
    /// The zone tip and origin share no history.
    Unrelated { zone: String, origin: Oid, tip: Oid },
    /// No squash of the publishing zone on origin's line carries a Mortar-Squash-Of.
    NoSquashOf { zone: String },
    /// The publishing zone's squash names a zone tip that the zone branch holds neither itself nor as a recorded
    /// replay of it (`refs/mortar/replayed/<zone>/<source>`).
    SquashOfNotInZone { zone: String, squash_of: Oid },
}
