//! status_fields: the --json members in the spec's order; overlap_paths is every shared path once, sorted.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};
use snapshot::RefusedFile;

use super::{status_fields, StatusState};
use crate::join::{MoveCause, PendingMove};
use crate::report::OutValue;

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

fn texts(names: &[&str]) -> OutValue {
    OutValue::Texts(names.iter().map(|name| name.to_string()).collect())
}

fn text(value: &str) -> OutValue {
    OutValue::Text(value.to_string())
}

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn bare() -> StatusState {
    StatusState {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        paths: StatusPaths::default(),
        saved: true,
        refused: Vec::new(),
        zone_moved: false,
        joined: true,
        no_work_yet: false,
        paused: None,
        stalled: None,
        overlaps: Vec::new(),
        gaps: Vec::new(),
    }
}

fn named(fields: Vec<(String, OutValue)>) -> Vec<(&'static str, OutValue)> {
    const NAMES: [&str; 17] = [
        "zone",
        "request",
        "changed",
        "untracked",
        "conflicted",
        "saved",
        "refused",
        "cap",
        "zone_moved",
        "joined",
        "paused_commit",
        "paused_files",
        "stalled_from",
        "stalled_to",
        "overlaps",
        "overlap_paths",
        "gaps",
    ];
    assert_eq!(fields.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(), NAMES.to_vec());
    NAMES.into_iter().zip(fields.into_iter().map(|(_, value)| value)).collect()
}

#[test]
fn a_bare_state_has_every_member_with_nulls_and_empty_lists() {
    assert_eq!(
        named(status_fields(&bare())),
        vec![
            ("zone", text("core")),
            ("request", text("fix-1")),
            ("changed", texts(&[])),
            ("untracked", texts(&[])),
            ("conflicted", texts(&[])),
            ("saved", OutValue::Bool(true)),
            ("refused", texts(&[])),
            ("cap", OutValue::Null),
            ("zone_moved", OutValue::Bool(false)),
            ("joined", OutValue::Bool(true)),
            ("paused_commit", OutValue::Null),
            ("paused_files", texts(&[])),
            ("stalled_from", OutValue::Null),
            ("stalled_to", OutValue::Null),
            ("overlaps", texts(&[])),
            ("overlap_paths", texts(&[])),
            ("gaps", texts(&[])),
        ]
    );
}

#[test]
fn a_full_state_fills_every_member() {
    let state = StatusState {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        paths: StatusPaths {
            changed: paths(&["a.rs"]),
            untracked: paths(&["n.txt", "m.txt"]),
            conflicted: paths(&["c"]),
        },
        saved: false,
        refused: vec![
            RefusedFile { path: PathBuf::from("big.bin"), size: 900, cap: 512 },
            RefusedFile { path: PathBuf::from("huge.bin"), size: 9000, cap: 256 },
        ],
        zone_moved: true,
        joined: false,
        no_work_yet: false,
        paused: Some(PausedJoin { commit: oid("3333333333333333333333333333333333333333"), files: paths(&["c"]) }),
        stalled: Some(PendingMove {
            request_ref: "refs/heads/req/core/fix-1".to_string(),
            old: oid("1111111111111111111111111111111111111111"),
            new: oid("2222222222222222222222222222222222222222"),
            op_id: "op-1".to_string(),
            folder_tree: None,
            cause: MoveCause::Rebase,
        }),
        overlaps: vec![
            ("fix-3".to_string(), paths(&["z.rs", "b.rs"])),
            ("fix-2".to_string(), paths(&["b.rs", "a.rs"])),
        ],
        gaps: vec!["docs".to_string()],
    };
    assert_eq!(
        named(status_fields(&state)),
        vec![
            ("zone", text("core")),
            ("request", text("fix-1")),
            ("changed", texts(&["a.rs"])),
            ("untracked", texts(&["n.txt", "m.txt"])),
            ("conflicted", texts(&["c"])),
            ("saved", OutValue::Bool(false)),
            ("refused", texts(&["big.bin", "huge.bin"])),
            ("cap", OutValue::Number(512)),
            ("zone_moved", OutValue::Bool(true)),
            ("joined", OutValue::Bool(false)),
            ("paused_commit", text("3333333333333333333333333333333333333333")),
            ("paused_files", texts(&["c"])),
            ("stalled_from", text("1111111111111111111111111111111111111111")),
            ("stalled_to", text("2222222222222222222222222222222222222222")),
            ("overlaps", texts(&["fix-3", "fix-2"])),
            ("overlap_paths", texts(&["a.rs", "b.rs", "z.rs"])),
            ("gaps", texts(&["docs"])),
        ]
    );
}
