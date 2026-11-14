//! Display for PublishError's own refusals (PushRefused, RemoteMissing), each as literal text naming what it carries
//! and saying nothing was published. The Land arm, which prints LandError's Oid text, is a Nagatha RIT (owed,
//! S-101); the State arm, which runs LandingStateError's Display, is tested in tests/publish_zit (state_display); the
//! Squash, Git and Oplog arms embed another error's text and have no case here.

use super::PublishError;
use gitio::PushRefusal;

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
