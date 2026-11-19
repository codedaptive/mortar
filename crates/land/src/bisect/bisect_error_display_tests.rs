//! Failure texts for the variants whose text is bisect's own alone: OriginMissing. NotOnOrigin, OffTheLine and
//! ProbeCouldNotRun print a commit (gitio's Oid Display), and Git and Culprit print their inner error, so their
//! texts are repo integration tests (RITs): tests/land_rit/owed_displays.rs covers NotOnOrigin, OffTheLine and
//! ProbeCouldNotRun; Culprit's CulpritError text is covered there (NotALanding) and in tests/land_rit/displays.rs
//! (Trailers); Git's is not yet written.

use super::BisectError;

#[test]
fn origin_missing_names_the_ref_and_says_to_name_a_local_branch() {
    let err = BisectError::OriginMissing { refname: "refs/heads/main".to_string() };
    assert_eq!(
        err.to_string(),
        "refs/heads/main does not exist in this repository; name an origin branch that exists here as a local \
         branch, then bisect again"
    );
}
