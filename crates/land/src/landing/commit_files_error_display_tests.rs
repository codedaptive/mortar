//! Display for CommitFilesError: NotLanding, NotLanded and FolderOffTip (a folder commit and an unborn folder) as
//! literal text;
//! Paths and Land show exactly the inner error's text.

use super::{CommitFilesError, LandError};
use gitio::Oid;
use snapshot::CommitPathsError;
use std::path::PathBuf;

const TIP: &str = "2222222222222222222222222222222222222222";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn absolute() -> CommitPathsError {
    CommitPathsError::Absolute { path: PathBuf::from("/outside") }
}

fn no_zone() -> LandError {
    LandError::NoZone { commit: oid("1111111111111111111111111111111111111111") }
}

#[test]
fn commit_files_error_display_says_why_or_passes_the_inner_text_through() {
    let rows = [
        (
            "not landing",
            CommitFilesError::NotLanding.to_string(),
            "there is no landing in flight: nothing landed since the last publish, nothing written".to_string(),
        ),
        (
            "not landed",
            CommitFilesError::NotLanded {
                landed: oid("1111111111111111111111111111111111111111"),
                integration: oid(TIP),
            }
            .to_string(),
            "integration is at 2222222222222222222222222222222222222222, not at \
             1111111111111111111111111111111111111111, the commit the landing state names; nothing was written"
                .to_string(),
        ),
        (
            "folder off tip",
            CommitFilesError::FolderOffTip {
                folder: Some(oid("1111111111111111111111111111111111111111")),
                integration: oid("2222222222222222222222222222222222222222"),
            }
            .to_string(),
            "the integration folder is at 1111111111111111111111111111111111111111, not at integration's tip \
             2222222222222222222222222222222222222222; nothing written"
                .to_string(),
        ),
        (
            "folder unborn",
            CommitFilesError::FolderOffTip { folder: None, integration: oid(TIP) }.to_string(),
            "the integration folder is at no commit, not at integration's tip \
             2222222222222222222222222222222222222222; nothing written"
                .to_string(),
        ),
        ("paths", CommitFilesError::Paths(absolute()).to_string(), absolute().to_string()),
        ("land", CommitFilesError::Land(no_zone()).to_string(), no_zone().to_string()),
    ];
    for (name, text, expected) in rows {
        assert_eq!(text, expected, "{name}");
    }
}
