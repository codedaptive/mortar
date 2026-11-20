use super::refusal_ok;

const REFUSED: &str = "only Mortar moves refs/heads/main\n\
                       owner: the landing surface: mr land and mr publish\n\
                       next: bring your work in with `mr join`, or ask for a landing\n\
                       fatal: ref updates aborted by hook\n";

#[test]
fn the_guards_refusal_of_the_named_ref_is_ok() {
    assert!(refusal_ok(REFUSED, "only Mortar moves refs/heads/main"));
}

#[test]
fn another_why_is_not() {
    assert!(!refusal_ok(REFUSED, "only Mortar moves refs/heads/integration"));
    let unreadable = "cannot read the reference-transaction hook's input line \"0 ref:refs/heads/main HEAD\"\n\
                      owner: the landing surface: mr land and mr publish\n\
                      next: bring your work in with `mr join`, or ask for a landing\n";
    assert!(!refusal_ok(unreadable, "only Mortar moves refs/heads/main"));
}

#[test]
fn another_owner_or_next_is_not() {
    let owner = "only Mortar moves refs/heads/main\nowner: the harness that installs the guard\n\
                 next: bring your work in with `mr join`, or ask for a landing\n";
    assert!(!refusal_ok(owner, "only Mortar moves refs/heads/main"));
    let next = "only Mortar moves refs/heads/main\nowner: the landing surface: mr land and mr publish\n\
                next: mr guard install --actor <name>\n";
    assert!(!refusal_ok(next, "only Mortar moves refs/heads/main"));
}

#[test]
fn nothing_said_is_not() {
    assert!(!refusal_ok("", "only Mortar moves refs/heads/main"));
    assert!(!refusal_ok("fatal: ref updates aborted by hook\n", "only Mortar moves refs/heads/main"));
}
