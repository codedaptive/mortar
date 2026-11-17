//! restore_why: each JoinRunError a restore of a paused join can give has its own why lines, built from its fields;
//! the errors a restore never gives share one line. The Git case is the RIT in cli_rit/restore_git_errors.rs (S-101).

use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use ::join::JoinRunError;
use gitio::Oid;

use super::restore_why;

const STATE: &str = "/w/.git/mortar-join.json";
const NEVER: &str = "join answered an error it never gives for a restore of a paused join";

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn why(error: JoinRunError) -> Vec<String> {
    restore_why(&error)
}

#[test]
fn a_named_op_that_records_no_paused_join_is_named() {
    assert_eq!(
        why(JoinRunError::NotAResolveOp { op_id: "op-9".to_string() }),
        vec!["op op-9 records no paused join that undo can put back"]
    );
}

#[test]
fn a_recorded_state_that_cannot_be_read_back_gives_the_op_and_the_reason() {
    let error = JoinRunError::RecordedStateCorrupt { op_id: "op-9".to_string(), reason: "no files key".to_string() };
    assert_eq!(why(error), vec!["the paused join op op-9 recorded cannot be read back: no files key"]);
}

#[test]
fn another_paused_join_in_the_state_file_is_never_overwritten() {
    let error = JoinRunError::StatePresent { state: PathBuf::from(STATE) };
    assert_eq!(why(error), vec!["/w/.git/mortar-join.json holds another paused join, which undo never overwrites"]);
}

#[test]
fn a_state_file_mortar_did_not_write_is_named_and_kept() {
    let error = JoinRunError::StateCorrupt { path: PathBuf::from(STATE), reason: "commit is not an oid".to_string() };
    assert_eq!(
        why(error),
        vec![
            "/w/.git/mortar-join.json is not a paused-join file mortar wrote: commit is not an oid",
            "the file is kept",
        ]
    );
}

#[test]
fn a_state_file_that_cannot_be_read_gives_the_io_error() {
    let error = JoinRunError::StateRead { path: PathBuf::from(STATE), source: io::Error::other("boom") };
    assert_eq!(why(error), vec!["/w/.git/mortar-join.json could not be read: boom"]);
}

#[test]
fn a_state_file_that_cannot_be_written_gives_the_io_error() {
    let source = io::Error::new(io::ErrorKind::NotFound, "gone");
    let error = JoinRunError::StateWrite { path: PathBuf::from("/w/.git/mortar-join.json.tmp"), source };
    assert_eq!(why(error), vec!["/w/.git/mortar-join.json.tmp could not be written: gone"]);
}

#[test]
fn a_conflicted_path_that_is_not_utf8_is_shown_lossily() {
    let error = JoinRunError::StatePath(PathBuf::from(OsStr::from_bytes(b"src/bad\xffname.rs")));
    assert_eq!(
        why(error),
        vec!["the conflicted path src/bad\u{FFFD}name.rs is not UTF-8, so the paused-join file cannot hold it"]
    );
}

#[test]
fn a_request_ref_moved_since_the_pause_names_where_it_is() {
    let error = JoinRunError::RestoreStale {
        request_ref: "refs/heads/req/a".to_string(),
        expected: oid('a'),
        actual: Some(oid('b')),
    };
    assert_eq!(
        why(error),
        vec![
            "refs/heads/req/a is at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, not at \
             aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, where the paused join stopped"
        ]
    );
}

#[test]
fn a_request_ref_that_is_gone_is_at_nothing() {
    let error =
        JoinRunError::RestoreStale { request_ref: "refs/heads/req/a".to_string(), expected: oid('a'), actual: None };
    assert_eq!(
        why(error),
        vec!["refs/heads/req/a is at nothing, not at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, where the paused join stopped"]
    );
}

#[test]
fn errors_a_restore_never_gives_share_one_line() {
    for error in [
        JoinRunError::ZoneMissing("refs/heads/zone/cli".to_string()),
        JoinRunError::NothingToResolve { state: PathBuf::from(STATE) },
        JoinRunError::StillMarked(Vec::new()),
        JoinRunError::ConflictedPathIsFolder { path: PathBuf::from("src") },
    ] {
        assert_eq!(why(error), vec![NEVER]);
    }
}
