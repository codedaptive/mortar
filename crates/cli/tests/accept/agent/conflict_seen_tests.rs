use super::conflict_seen;

const SAVE: &str = "1\t0\tsh\tmr\tsave\tr1: work\n";
const DRIVE: &str = "5\t5\tdrive\tmr\tsave\trc: conflict\n5\t5\tdrive\tmr\tjoin\n";
const JOIN: &str = "2\t0\tsh\tmr\tjoin\n";
const RESOLVE: &str = "3\t0\tsh\tmr\tresolve\n";

#[test]
fn the_drives_join_logged_before_the_agents_first_join_and_a_resolve_is_seen() {
    assert_eq!(conflict_seen(&[SAVE, DRIVE, JOIN, RESOLVE, RESOLVE, JOIN].concat()), Ok(()));
}

#[test]
fn an_agent_that_never_ran_mr_save_still_meets_the_conflict() {
    assert_eq!(conflict_seen(&[DRIVE, JOIN, RESOLVE, JOIN].concat()), Ok(()));
}

#[test]
fn no_drive_join_is_named_first() {
    let error = conflict_seen(&[SAVE, JOIN, RESOLVE].concat()).unwrap_err();
    assert_eq!(error, "the drive's mr join in rc never ran");
}

#[test]
fn an_agent_join_logged_before_the_drives_is_an_error() {
    let error = conflict_seen(&[SAVE, JOIN, DRIVE, RESOLVE].concat()).unwrap_err();
    assert_eq!(error, "the agent's first mr join was logged before the drive's, so it met no conflict");
}

#[test]
fn no_agent_join_or_no_resolve_is_named() {
    assert_eq!(conflict_seen(&[SAVE, DRIVE, RESOLVE].concat()).unwrap_err(), "the agent ran no mr join");
    assert_eq!(conflict_seen(&[SAVE, DRIVE, JOIN].concat()).unwrap_err(), "the agent ran no mr resolve");
}

#[test]
fn a_join_or_resolve_from_a_hook_is_not_the_agents() {
    let hook = "6\t4\tgit\tmr\tjoin\n7\t4\tgit\tmr\tresolve\n";
    assert_eq!(conflict_seen(&[SAVE, DRIVE, hook].concat()).unwrap_err(), "the agent ran no mr join");
}
