//! decide_commit_files: every branch with literal ids: no state; the state at the tip; a pending commit at the tip
//! (resolved: the next commit goes on top of it); a stale pending commit with integration at the tip (dropped); the
//! state naming another commit (integration at origin, and elsewhere); the folder off the tip and unborn.

use super::{decide_commit_files, CommitFilesError, LandingMember, LandingState};
use gitio::Oid;
use oplog::ChangeId;

const ORIGIN: &str = "1111111111111111111111111111111111111111";
const LANDED: &str = "2222222222222222222222222222222222222222";
const PENDING: &str = "3333333333333333333333333333333333333333";
const STALE: &str = "5555555555555555555555555555555555555555";
const OTHER: &str = "6666666666666666666666666666666666666666";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(integration: &str, pending: Option<&str>) -> LandingState {
    LandingState {
        origin_at_start: oid(ORIGIN),
        earlier: Vec::new(),
        newest: LandingMember {
            change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap(),
            zone: "land".to_string(),
            squash: oid("4444444444444444444444444444444444444444"),
            integration: oid(integration),
        },
        pending: pending.map(oid),
    }
}

/// What a decision came to, comparable: CommitFilesError is not Eq.
#[derive(Debug, PartialEq)]
enum Seen {
    Ok(LandingState),
    NotLanding,
    FolderOffTip { folder: Option<Oid>, integration: Oid },
    NotLanded { landed: Oid, integration: Oid },
    Other(String),
}

fn seen(decision: Result<LandingState, CommitFilesError>) -> Seen {
    match decision {
        Ok(state) => Seen::Ok(state),
        Err(CommitFilesError::NotLanding) => Seen::NotLanding,
        Err(CommitFilesError::FolderOffTip { folder, integration }) => Seen::FolderOffTip { folder, integration },
        Err(CommitFilesError::NotLanded { landed, integration }) => Seen::NotLanded { landed, integration },
        Err(other) => Seen::Other(format!("{other:?}")),
    }
}

#[test]
fn decide_commit_files_needs_a_landing_at_the_tip_and_the_folder_there() {
    let rows = [
        ("no state", None, LANDED, Some(LANDED), Seen::NotLanding),
        ("state at the tip", Some(state(LANDED, None)), LANDED, Some(LANDED), Seen::Ok(state(LANDED, None))),
        (
            "pending at the tip",
            Some(state(LANDED, Some(PENDING))),
            PENDING,
            Some(PENDING),
            Seen::Ok(state(PENDING, None)),
        ),
        (
            "stale pending, integration at the tip",
            Some(state(LANDED, Some(STALE))),
            LANDED,
            Some(LANDED),
            Seen::Ok(state(LANDED, None)),
        ),
        (
            "integration at origin, a land killed before its op",
            Some(state(LANDED, None)),
            ORIGIN,
            Some(ORIGIN),
            Seen::NotLanded { landed: oid(LANDED), integration: oid(ORIGIN) },
        ),
        (
            "integration elsewhere",
            Some(state(LANDED, None)),
            OTHER,
            Some(OTHER),
            Seen::NotLanded { landed: oid(LANDED), integration: oid(OTHER) },
        ),
        (
            "folder off the tip",
            Some(state(LANDED, None)),
            LANDED,
            Some(ORIGIN),
            Seen::FolderOffTip { folder: Some(oid(ORIGIN)), integration: oid(LANDED) },
        ),
        (
            "folder unborn",
            Some(state(LANDED, None)),
            LANDED,
            None,
            Seen::FolderOffTip { folder: None, integration: oid(LANDED) },
        ),
    ];
    for (name, given, tip, folder, expected) in rows {
        let folder = folder.map(oid);
        assert_eq!(seen(decide_commit_files(given, &oid(tip), folder.as_ref())), expected, "{name}");
    }
}
