//! sweep_refs against attempts' trysnap refs in a throwaway repository: the cases src/sweep/sweep_refs.rs's header
//! names for them. A request is closed when its request branch and every attempt branch are gone.
use super::attempt_support::{trysnap_ref, Attempts, REQ_REF};
use super::maintenance_support::{assert_delete_pair, caller, records, refs, sweep, ACTOR, SNAP};
use crate::support::{run, Fixture};

const OPEN_SNAP: &str = "refs/mortar/snap/red/open";
const OPEN_TRYSNAP: &str = "refs/mortar/trysnap/red/open/1";
const OPEN_TRY: &str = "refs/heads/try/red/open/1";
const CLOSED: [&str; 3] = ["refs/mortar/snap/red/closed", "refs/mortar/trysnap/red/closed/1", "refs/mortar/trysnap/red/closed/2"];
const HAND_MADE: &str = "refs/heads/try/red/one/01";

#[test]
fn a_closed_requests_trysnap_refs_go_with_its_snap_ref_and_open_requests_keep_theirs() {
    let a = Attempts::with(1);
    a.trysnaps(1);
    let head = a.f.head.clone();
    run(&a.f.git, &["update-ref", SNAP, head.as_str()]);
    for name in [OPEN_SNAP, OPEN_TRYSNAP, OPEN_TRY].into_iter().chain(CLOSED) {
        run(&a.f.git, &["update-ref", name, head.as_str()]);
    }
    let mut expected = refs(&a.f.git);
    for name in CLOSED { expected.remove(name); }
    let start = records(&a.f).len();
    assert_eq!(sweep(&a.f, &a.f.git, &[], &[], &[]).unwrap(), CLOSED.map(str::to_string));
    assert_eq!(refs(&a.f.git), expected);
    for kept in [SNAP, &trysnap_ref(1), REQ_REF, OPEN_SNAP, OPEN_TRYSNAP] {
        assert_eq!(a.f.git.read_ref(kept).unwrap(), Some(head.clone()), "{kept}");
    }
    assert_delete_pair(&records(&a.f)[start..], "folder-sweep-refs", &CLOSED, &head);
}

#[test]
fn after_drop_attempts_and_remove_the_trysnap_refs_left_are_swept() {
    let a = Attempts::with(2);
    a.trysnaps(2);
    let head = a.f.head.clone();
    let saved = [(1, true), (2, true)];
    let dropped = caller(&a.f.git, &a.f.common, || folders::drop_attempts(&a.f.git, "red", "one", &saved, ACTOR));
    assert_eq!(dropped.unwrap(), vec![1, 2]);
    a.remove_request().unwrap();
    assert_eq!(a.f.git.read_ref(&trysnap_ref(1)).unwrap(), Some(head.clone()), "remove leaves trysnap refs");
    let start = records(&a.f).len();
    let swept = [trysnap_ref(1), trysnap_ref(2)];
    assert_eq!(sweep(&a.f, &a.f.git, &[], &[], &[]).unwrap(), swept);
    for name in &swept { assert_eq!(a.f.git.read_ref(name).unwrap(), None, "{name}"); }
    assert_delete_pair(&records(&a.f)[start..], "folder-sweep-refs", &[swept[0].as_str(), swept[1].as_str()], &head);
}

#[test]
fn with_only_a_hand_made_try_ref_that_is_not_an_attempt_left_the_snap_and_trysnap_refs_are_swept_and_it_is_untouched() {
    let f = Fixture::new();
    let (one, two) = (trysnap_ref(1), trysnap_ref(2));
    let swept = [SNAP, one.as_str(), two.as_str()];
    for name in swept.into_iter().chain([HAND_MADE]) { run(&f.git, &["update-ref", name, f.head.as_str()]); }
    let start = records(&f).len();
    assert_eq!(sweep(&f, &f.git, &[], &[], &[]).unwrap(), swept.map(str::to_string));
    for name in swept { assert_eq!(f.git.read_ref(name).unwrap(), None, "{name}"); }
    assert_eq!(f.git.read_ref(HAND_MADE).unwrap(), Some(f.head.clone()));
    assert_delete_pair(&records(&f)[start..], "folder-sweep-refs", &swept, &f.head);
}
