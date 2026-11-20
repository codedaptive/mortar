//! replay_commits: an empty list is clean at start with no entries and no git call.

use crate::replay_commits::replay_commits;
use crate::replay_git_fake::ReplayGitFake;
use crate::{MappedReplay, ReplayResult};
use gitio::Oid;

const START: &str = "2222222222222222222222222222222222222222";
const ONTO: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn an_empty_list_is_clean_at_start_with_no_git_call() {
    let git = ReplayGitFake::default();
    assert_eq!(
        replay_commits(&git, &oid(ONTO), &oid(START), &[]),
        Ok(MappedReplay {
            result: ReplayResult::Clean(oid("2222222222222222222222222222222222222222")),
            mapped: Vec::new(),
        })
    );
    assert!(git.calls.borrow().is_empty());
}
