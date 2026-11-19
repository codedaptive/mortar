//! Every way bisect can fail before or while it searches.
//! A bare type; bisect_error_source_tests exercise each variant, and bisect_error_display_tests the OriginMissing
//! text. The other arms' texts are repo integration tests (RITs): tests/land_rit/owed_displays.rs covers
//! NotOnOrigin, OffTheLine and ProbeCouldNotRun; Culprit prints CulpritError's text, covered there (NotALanding)
//! and in tests/land_rit/displays.rs (Trailers); Git's is not yet written.

use gitio::{GitError, Oid};

use super::CulpritError;

/// A failure of the top-level bisect: a refusal before any probe, a probe that could not run, or an inner error.
#[derive(Debug)]
pub enum BisectError {
    /// The origin branch ref does not exist.
    OriginMissing { refname: String },
    /// The failed commit is not on the origin branch, so it names no landing.
    NotOnOrigin { failed: Oid, origin_branch: String },
    /// `last_pass` is not on `failed`'s first-parent line, so the range names no landings.
    OffTheLine { last_pass: Oid, failed: Oid },
    /// The caller's command could not be run at `commit` (spawn, wait or stderr read failed): the suite's verdict
    /// there is unknown, so bisect stops. `error` is the failure's text.
    ProbeCouldNotRun { commit: Oid, error: String },
    /// A git step failed.
    Git(GitError),
    /// The culprit's landing could not be read.
    Culprit(CulpritError),
}
