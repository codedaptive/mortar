//! status_report: a clean, saved, joined folder and a folder with every part, rendered both ways. The same two
//! folders as the whole Report are the ZUTs in src/status/status_report_tests.rs.
//! ZIT (S-79 as replaced): status_report calls status_lines, status_fields, status_next and status_outcome for real;
//! render_plain and render_json render both forms.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};
use snapshot::RefusedFile;

use cli::{render_json, render_plain, status_report, MoveCause, PendingMove, StatusState};

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const PAUSED: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

fn clean() -> StatusState {
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

fn full() -> StatusState {
    StatusState {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        paths: StatusPaths {
            changed: paths(&["src/a.rs", "src/b.rs"]),
            untracked: paths(&["notes.txt"]),
            conflicted: paths(&["src/c.rs"]),
        },
        saved: false,
        refused: vec![RefusedFile { path: PathBuf::from("data/big.bin"), size: 2_000_000, cap: 1_048_576 }],
        zone_moved: true,
        joined: false,
        no_work_yet: false,
        paused: Some(PausedJoin { commit: oid(PAUSED), files: paths(&["src/c.rs", "src/d.rs"]) }),
        stalled: Some(PendingMove {
            request_ref: "refs/heads/req/core/fix-1".to_string(),
            old: oid(OLD),
            new: oid(NEW),
            op_id: "op-7".to_string(),
            folder_tree: None,
            cause: MoveCause::Rebase,
        }),
        overlaps: vec![
            ("fix-2".to_string(), paths(&["src/b.rs"])),
            ("fix-3".to_string(), paths(&["src/a.rs", "src/b.rs"])),
        ],
        gaps: vec!["docs/guide.txt".to_string(), "tools/gen.rs".to_string()],
    }
}

#[test]
fn a_clean_folder_renders_plain_and_json() {
    let report = status_report(&clean());
    assert_eq!(
        render_plain(&report),
        "request fix-1 in zone core\n\
         no changed, untracked or conflicted paths\n\
         everything is saved\n\
         the zone has no commits this request has not taken in\n\
         the request is joined to the zone\n\
         next: wait for the zone's tests\n"
    );
    assert_eq!(
        render_json(&report),
        "{\"verb\":\"status\",\"ok\":true,\"outcome\":\"joined\",\"exit\":0,\"next\":\"wait for the zone's tests\",\
         \"lines\":[\"request fix-1 in zone core\",\"no changed, untracked or conflicted paths\",\
         \"everything is saved\",\"the zone has no commits this request has not taken in\",\
         \"the request is joined to the zone\"],\"owner\":null,\"zone\":\"core\",\"request\":\"fix-1\",\
         \"changed\":[],\"untracked\":[],\"conflicted\":[],\"saved\":true,\"refused\":[],\"cap\":null,\
         \"zone_moved\":false,\"joined\":true,\"paused_commit\":null,\"paused_files\":[],\"stalled_from\":null,\
         \"stalled_to\":null,\"overlaps\":[],\"overlap_paths\":[],\"gaps\":[]}\n"
    );
}

#[test]
fn a_full_folder_renders_plain_and_json() {
    let report = status_report(&full());
    assert_eq!(
        render_plain(&report),
        "request fix-1 in zone core\n\
         changed: src/a.rs, src/b.rs\n\
         untracked: notes.txt\n\
         conflicted: src/c.rs\n\
         there is work not yet saved\n\
         not saved: data/big.bin is 2000000 bytes, over the size cap of 1048576 bytes\n\
         the zone has commits this request has not taken in\n\
         a join is paused at commit 3333333333333333333333333333333333333333 with conflicted files: src/c.rs, \
         src/d.rs\n\
         a folder move is stalled: refs/heads/req/core/fix-1 stands at 2222222222222222222222222222222222222222, \
         the folder at 1111111111111111111111111111111111111111\n\
         overlaps with fix-2: src/b.rs\n\
         overlaps with fix-3: src/a.rs, src/b.rs\n\
         gaps (needed outside the work order): docs/guide.txt, tools/gen.rs\n\
         next: mr join\n"
    );
    assert_eq!(
        render_json(&report),
        "{\"verb\":\"status\",\"ok\":true,\"outcome\":\"stalled\",\"exit\":0,\"next\":\"mr join\",\
         \"lines\":[\"request fix-1 in zone core\",\"changed: src/a.rs, src/b.rs\",\"untracked: notes.txt\",\
         \"conflicted: src/c.rs\",\"there is work not yet saved\",\
         \"not saved: data/big.bin is 2000000 bytes, over the size cap of 1048576 bytes\",\
         \"the zone has commits this request has not taken in\",\
         \"a join is paused at commit 3333333333333333333333333333333333333333 with conflicted files: src/c.rs, \
         src/d.rs\",\
         \"a folder move is stalled: refs/heads/req/core/fix-1 stands at 2222222222222222222222222222222222222222, \
         the folder at 1111111111111111111111111111111111111111\",\
         \"overlaps with fix-2: src/b.rs\",\"overlaps with fix-3: src/a.rs, src/b.rs\",\
         \"gaps (needed outside the work order): docs/guide.txt, tools/gen.rs\"],\"owner\":null,\
         \"zone\":\"core\",\"request\":\"fix-1\",\"changed\":[\"src/a.rs\",\"src/b.rs\"],\
         \"untracked\":[\"notes.txt\"],\"conflicted\":[\"src/c.rs\"],\"saved\":false,\"refused\":[\"data/big.bin\"],\
         \"cap\":1048576,\"zone_moved\":true,\"joined\":false,\
         \"paused_commit\":\"3333333333333333333333333333333333333333\",\"paused_files\":[\"src/c.rs\",\"src/d.rs\"],\
         \"stalled_from\":\"1111111111111111111111111111111111111111\",\
         \"stalled_to\":\"2222222222222222222222222222222222222222\",\"overlaps\":[\"fix-2\",\"fix-3\"],\
         \"overlap_paths\":[\"src/a.rs\",\"src/b.rs\"],\"gaps\":[\"docs/guide.txt\",\"tools/gen.rs\"]}\n"
    );
}
