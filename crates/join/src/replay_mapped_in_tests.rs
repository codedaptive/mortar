use crate::fake_call::FakeCall;
use crate::replay_git_fake::ReplayGitFake;
use crate::replay_mapped_in::replay_mapped_in;
use crate::{JoinError, MappedReplay, ReplayResult};
use gitio::Oid;

const BASE: &str = "1111111111111111111111111111111111111111";
const ONTO: &str = "3333333333333333333333333333333333333333";
const TIP_OID: &str = "2222222222222222222222222222222222222222";
const TIP: &str = "refs/heads/topic";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// onto present, the tip ref at TIP_OID, an empty range, and git's replay scripted as available, so a call to it
/// would show in the log.
fn fake() -> ReplayGitFake {
    let mut git = ReplayGitFake {
        available: Some(true),
        rev_parse: Some(Ok(Some(oid(ONTO)))),
        rev_list: Some(Vec::new()),
        ..ReplayGitFake::default()
    };
    git.refs.insert(TIP.to_string(), oid(TIP_OID));
    git
}

#[test]
fn missing_onto_is_refused_after_only_the_rev_parse() {
    let git = ReplayGitFake { rev_parse: Some(Ok(None)), ..fake() };
    assert_eq!(replay_mapped_in(&git, &oid(BASE), TIP, &oid(ONTO)), Err(JoinError::OntoMissing(oid(ONTO))));
    assert_eq!(*git.calls.borrow(), vec![FakeCall::RevParse(ONTO.to_string())]);
}

#[test]
fn missing_tip_is_refused_with_its_ref_name_before_the_rev_list() {
    let mut git = fake();
    git.refs.clear();
    assert_eq!(
        replay_mapped_in(&git, &oid(BASE), TIP, &oid(ONTO)),
        Err(JoinError::TipMissing("refs/heads/topic".to_string()))
    );
    assert_eq!(
        *git.calls.borrow(),
        vec![FakeCall::RevParse(ONTO.to_string()), FakeCall::ReadRef("refs/heads/topic".to_string())]
    );
}

#[test]
fn an_empty_range_is_clean_at_onto_with_no_entries_and_never_asks_for_git_replay() {
    let git = fake();
    assert_eq!(
        replay_mapped_in(&git, &oid(BASE), TIP, &oid(ONTO)),
        Ok(MappedReplay { result: ReplayResult::Clean(oid(ONTO)), mapped: Vec::new() })
    );
    assert_eq!(
        *git.calls.borrow(),
        vec![
            FakeCall::RevParse(ONTO.to_string()),
            FakeCall::ReadRef("refs/heads/topic".to_string()),
            FakeCall::RevList { base: oid(BASE), tip: oid(TIP_OID) },
        ]
    );
}
