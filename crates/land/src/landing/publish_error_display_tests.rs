//! Display for PublishError's own refusals (PushRefused, RemoteMissing, PushIntent), each as literal text naming what
//! it carries and saying nothing was published or pushed; PushIntent's OpPhase is built as data and not printed. The
//! Land arm, which prints LandError's Oid text, is a Nagatha RIT (owed, S-101); the State arm, which runs
//! LandingStateError's Display, is tested in tests/publish_zit (state_display); the Squash, Git and Oplog arms embed
//! another error's text and have no case here.

use super::PublishError;
use gitio::PushRefusal;
use oplog::OpPhase;

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
    let text = "the push intent op-7 for this move was already open or already done when publish opened it, so \
                nothing was pushed; next: check the remote's origin branch, then mr publish";
    for phase in [OpPhase::Intent, OpPhase::Done] {
        assert_eq!(PublishError::PushIntent { op_id: "op-7".to_string(), phase }.to_string(), text);
    }
}
