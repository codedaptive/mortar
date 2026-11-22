//! Display for SquashError: each test pins the whole Display text, in land's own words around the values the test
//! passes, except: the Git arm is gitio's text alone; the Oplog and ChangeId arms are land's prefix then oplog's text;
//! the Trailers arm is land's prefix then oplog's text with gitio's inside it.

use super::SquashError;
use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-squash:refs/mortar/squash/land:none:k:op-7".to_string(),
        kind: "land-squash".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

#[test]
fn ref_missing_text_is_the_ref_does_not_exist_then_no_commit_written_and_no_ref_moved() {
    let text = SquashError::RefMissing { refname: "refs/heads/zone/land".to_string() }.to_string();
    assert_eq!(text, "ref refs/heads/zone/land does not exist; no commit was written and no ref was moved");
}

#[test]
fn op_failed_text_is_the_op_its_key_failed_why_nothing_moved_then_what_to_do() {
    assert_eq!(
        SquashError::OpFailed(record(OpPhase::Failed)).to_string(),
        "op op-7 (key land-squash:refs/mortar/squash/land:none:k:op-7) failed: the squash ref was not at its expected \
         value, so nothing was moved; run squash again to read the refs afresh"
    );
}

#[test]
fn op_repeated_text_is_the_key_already_done_in_the_op_log_nothing_moved_then_what_to_do() {
    assert_eq!(
        SquashError::OpRepeated(record(OpPhase::Done)).to_string(),
        "op key land-squash:refs/mortar/squash/land:none:k:op-7 is already done in the op log, so this squash moved \
         nothing; run squash again"
    );
}

#[test]
fn attr_source_missing_text_is_the_commit_does_not_exist_merge_cannot_read_attributes_then_no_commit_written_and_no_ref_moved() {
    assert_eq!(SquashError::AttrSourceMissing { commit: a() }.to_string(), "attribute source commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa does not exist, so the merge cannot read its attributes; no commit was written and no ref was moved");
}

#[test]
fn unrelated_text_is_the_zone_tip_shares_no_history_with_origin_then_nothing_squashed_and_no_ref_moved() {
    assert_eq!(SquashError::Unrelated { origin: a(), source: b() }.to_string(), "zone tip bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb shares no history with origin aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; nothing was squashed and no ref was moved");
}

fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}
fn bad_id() -> ChangeIdError {
    ChangeIdError::Malformed { text: "bad".into(), reason: "expected 32 characters".into() }
}

#[test]
fn git_text_is_gitios_command_exit_code_and_stderr_with_no_prefix() {
    assert_eq!(SquashError::Git(git_error()).to_string(), "git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn oplog_text_is_the_squash_prefix_the_line_not_a_record_the_reason_mortar_did_not_write_it_then_what_to_do() {
    let inner = OplogError::Malformed { line: Some(7), reason: "missing op_id".into() };
    assert_eq!(SquashError::Oplog(inner).to_string(),
        "squash op log: op log line 7 is not a record (missing op_id); mortar did not write it, so check what did, then repair or remove that line of mortar/oplog.jsonl");
}

#[test]
fn trailers_text_is_the_squash_prefix_git_trailers_failed_then_gitios_command_exit_code_and_stderr() {
    let inner = MortarTrailerError::Git(git_error());
    assert_eq!(SquashError::Trailers(inner).to_string(), "squash trailers: git trailers failed: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn change_id_text_is_the_squash_prefix_not_a_change_id_the_text_then_the_reason() {
    assert_eq!(SquashError::ChangeId(bad_id()).to_string(), "squash change ID: not a change ID: \"bad\": expected 32 characters");
}

#[test]
fn trailer_missing_text_names_the_commit_and_the_trailer_then_what_to_do() {
    assert_eq!(SquashError::TrailerMissing { commit: a(), trailer: "Mortar-Change-Id".into() }.to_string(), "commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa has no Mortar-Change-Id trailer, so it is not a squash mortar made; name the commit at refs/mortar/squash/<zone>, or squash the zone again");
}

#[test]
fn trailer_malformed_text_names_the_commit_the_trailer_and_its_quoted_value_then_what_to_do() {
    assert_eq!(SquashError::TrailerMalformed { commit: a(), trailer: "Mortar-Change-Id".into(), value: "nope".into() }.to_string(), "commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa has a malformed Mortar-Change-Id trailer \"nope\", so it cannot be read as a squash; squash the zone again to write a new squash commit");
}

#[test]
fn requests_malformed_text_names_the_commit_and_the_request_lines_expected_then_what_to_do() {
    assert_eq!(SquashError::RequestsMalformed { commit: a() }.to_string(), "commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa's message does not list its requests as squash reads them (one `- <request>: ...` line each in the paragraph before the trailers, as many as a `<zone>: <n> requests` subject counts when the subject is squash's own); squash the zone again to write a new squash commit");
}
