//! repeat_step_for: an absent branch is Removed, one at the tip is Again, one anywhere else is Moved.

use gitio::Oid;

use super::repeat_step_for::repeat_step_for;
use super::RepeatStep;

const TIP: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn an_absent_branch_is_removed() {
    assert_eq!(repeat_step_for(None, &oid(TIP)), RepeatStep::Removed);
}

#[test]
fn a_branch_at_the_tip_is_again() {
    assert_eq!(repeat_step_for(Some(&oid(TIP)), &oid(TIP)), RepeatStep::Again);
}

#[test]
fn a_branch_at_another_commit_is_moved() {
    assert_eq!(repeat_step_for(Some(&oid(OTHER)), &oid(TIP)), RepeatStep::Moved);
}
