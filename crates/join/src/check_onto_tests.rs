use crate::check_onto::check_onto;
use crate::fake_call::FakeCall;
use crate::replay_git_fake::ReplayGitFake;
use crate::JoinError;
use gitio::{GitError, Oid};

const ONTO: &str = "1111111111111111111111111111111111111111";

fn onto() -> Oid {
    Oid::parse(ONTO).unwrap()
}

#[test]
fn present_onto_is_ok_after_one_rev_parse_of_its_id() {
    let git = ReplayGitFake { rev_parse: Some(Ok(Some(onto()))), ..ReplayGitFake::default() };
    assert_eq!(check_onto(&git, &onto()), Ok(()));
    assert_eq!(*git.calls.borrow(), vec![FakeCall::RevParse(ONTO.to_string())]);
}

#[test]
fn absent_onto_is_refused_naming_it() {
    let git = ReplayGitFake { rev_parse: Some(Ok(None)), ..ReplayGitFake::default() };
    assert_eq!(
        check_onto(&git, &onto()),
        Err(JoinError::OntoMissing(Oid::parse("1111111111111111111111111111111111111111").unwrap()))
    );
}

#[test]
fn failed_rev_parse_passes_through_as_git() {
    let error = GitError { args: vec!["rev-parse".to_string()], code: Some(128), stderr: "fatal: boom".to_string() };
    let git = ReplayGitFake { rev_parse: Some(Err(error.clone())), ..ReplayGitFake::default() };
    assert_eq!(check_onto(&git, &onto()), Err(JoinError::Git(error)));
}
