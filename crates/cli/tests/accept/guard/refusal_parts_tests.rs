use super::refusal_parts;

fn parts(why: &str, owner: &str, next: &str) -> (String, String, String) {
    (why.to_string(), owner.to_string(), next.to_string())
}

#[test]
fn reads_the_three_parts_among_gits_own_lines() {
    let stderr = "only Mortar moves refs/heads/integration\n\
                  owner: the landing surface: mr land and mr publish\n\
                  next: bring your work in with `mr join`, or ask for a landing\n\
                  fatal: update_ref failed for ref 'refs/heads/integration': ref updates aborted by hook\n";
    assert_eq!(
        refusal_parts(stderr),
        parts(
            "only Mortar moves refs/heads/integration",
            "the landing surface: mr land and mr publish",
            "bring your work in with `mr join`, or ask for a landing"
        )
    );
}

#[test]
fn why_is_the_line_just_before_the_owner() {
    let stderr = "hint: something git said\nonly Mortar pushes refs/heads/main to origin\nowner: o\nnext: n\n\
                  error: failed to push some refs to '/tmp/origin.git'\n";
    assert_eq!(refusal_parts(stderr), parts("only Mortar pushes refs/heads/main to origin", "o", "n"));
}

#[test]
fn no_owner_line_gives_nothing() {
    assert_eq!(refusal_parts("only Mortar moves refs/heads/main\nnext: n\n"), parts("", "", ""));
    assert_eq!(refusal_parts(""), parts("", "", ""));
}

#[test]
fn a_missing_why_or_next_is_empty() {
    assert_eq!(refusal_parts("owner: o\nfatal: aborted\n"), parts("", "o", ""));
}

#[test]
fn a_next_before_the_owner_is_not_read() {
    assert_eq!(refusal_parts("next: early\nwhy\nowner: o\n"), parts("why", "o", ""));
}
