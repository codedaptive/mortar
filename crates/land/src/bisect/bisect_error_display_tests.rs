//! Failure texts for the variants whose text is bisect's own alone: OriginMissing. NotOnOrigin, OffTheLine and
//! ProbeCouldNotRun print a commit (gitio's Oid Display), and Git and Culprit print their inner error, so their
//! texts are RITs owed to Nagatha (S-101).

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
