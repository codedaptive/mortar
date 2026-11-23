//! Display for PublishError's own refusals (PushRefused, RemoteMissing, PushIntent), each as literal text naming what
//! it carries and saying nothing was published or pushed; PushIntent chooses its text by OpPhase. The
//! Land and State arms prefix LandError's and LandingStateError's text with "publish: "; the Squash, Git and Oplog
//! arms embed another error's text and have no case here.

use super::{LandError, LandingStateError, PublishError};
use gitio::{Oid, PushRefusal};
use oplog::OpPhase;
use std::path::PathBuf;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }

fn refused(refusal: PushRefusal) -> String {
    PublishError::PushRefused { refname: "refs/heads/main".to_string(), refusal }.to_string()
}

#[test]
fn push_refused_names_the_ref_and_each_reason() {
    let tail = "; the remote's refs/heads/main is still where the landing started, so nothing was published";
    assert_eq!(
        refused(PushRefusal::StaleLease),
        format!("the remote refused the push of refs/heads/main (stale lease){tail}")
    );
    assert_eq!(
        refused(PushRefusal::NonFastForward),
        format!("the remote refused the push of refs/heads/main (not a fast-forward){tail}")
    );
    assert_eq!(
        refused(PushRefusal::Hook),
        format!("the remote refused the push of refs/heads/main (a hook on the remote declined it){tail}")
    );
    assert_eq!(
        refused(PushRefusal::Other("[remote rejected] (unpacker error)".to_string())),
        format!("the remote refused the push of refs/heads/main ([remote rejected] (unpacker error)){tail}")
    );
}

#[test]
fn remote_missing_names_the_remote_and_the_ref() {
    let error = PublishError::RemoteMissing { remote: "origin".to_string(), refname: "refs/heads/main".to_string() };
    assert_eq!(error.to_string(), "remote origin has no refs/heads/main to publish onto; nothing was published");
}

#[test]
fn push_intent_names_the_record_and_says_what_to_do_whether_open_or_done() {
    let rows = [
        (OpPhase::Intent, "the push intent op-7 for this move is still open: a publish was interrupted between its steps, so nothing was pushed or changed; next: mr publish, which reads the remote and closes it"),
        (OpPhase::Done, "the push intent op-7 for this move was already done when publish opened it, so nothing was pushed; next: check the remote's origin branch, then mr publish"),
    ];
    let failures: Vec<_> = rows.into_iter().filter_map(|(phase, text)| {
        let actual = PublishError::PushIntent { op_id: "op-7".to_string(), phase }.to_string();
        (actual != text).then(|| format!("{phase:?}: expected {text:?}, got {actual:?}"))
    }).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn land_is_prefixed_with_publish() {
    assert_eq!(PublishError::Land(LandError::NoZone { commit: a() }).to_string(), "publish: squash commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries no Mortar-Zone trailer, so its landing cannot name the zone; nothing was moved");
}

#[test]
fn state_is_prefixed_with_publish() {
    let inner = LandingStateError::Corrupt { path: PathBuf::from("/r/mortar/landing.json"), reason: "x".to_string() };
    let text = PublishError::State(inner).to_string();
    assert_eq!(text, "publish: landing state /r/mortar/landing.json is not a state land wrote: x");
}
