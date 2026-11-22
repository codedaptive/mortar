//! squash_message: the exact text for no request, one request and several, without and with a caller message.

use super::{squash_message, SquashRequest};

fn request(request: &str, subjects: &[&str]) -> SquashRequest {
    SquashRequest { request: request.to_string(), subjects: subjects.iter().map(|s| s.to_string()).collect() }
}

#[test]
fn no_requests_give_the_subject_line_alone() {
    assert_eq!(squash_message("land", &[], None), "land: 0 requests\n");
}

#[test]
fn one_request_gives_the_subject_a_blank_line_and_its_line() {
    assert_eq!(
        squash_message("land", &[request("r1", &["Add squash ref", "Add squash op"])], None),
        "land: 1 requests\n\n- r1: Add squash ref; Add squash op\n"
    );
}

#[test]
fn several_requests_give_one_line_each_in_order() {
    let requests = [request("r2", &["two a"]), request("r1", &["one a", "one b", "one c"]), request("r3", &["three"])];
    assert_eq!(
        squash_message("join", &requests, None),
        "join: 3 requests\n\n- r2: two a\n- r1: one a; one b; one c\n- r3: three\n"
    );
}

#[test]
fn every_request_line_starts_with_a_dash_and_a_space() {
    let message = squash_message("land", &[request("r1", &["one"]), request("r2", &["two"])], None);
    let lines: Vec<&str> = message.lines().skip(2).collect();
    assert_eq!(lines, vec!["- r1: one", "- r2: two"]);
    assert!(lines.iter().all(|line| line.starts_with("- ")), "{message:?}");
}

#[test]
fn a_caller_message_replaces_the_subject_and_keeps_the_request_lines() {
    let one = [request("r1", &["Add squash ref"])];
    let two = [request("r1", &["one"]), request("r2", &["two a", "two b"])];
    let rows: [(&str, &[SquashRequest], Option<&str>, &str); 5] = [
        ("subject only", &one, Some("Fix the guard"), "Fix the guard\n\n- r1: Add squash ref\n"),
        (
            "subject and body",
            &two,
            Some("Fix the guard\n\nWhy it was wrong."),
            "Fix the guard\n\nWhy it was wrong.\n\n- r1: one\n- r2: two a; two b\n",
        ),
        ("whitespace trimmed", &one, Some("  \n Fix the guard \n\n"), "Fix the guard\n\n- r1: Add squash ref\n"),
        ("no requests", &[], Some("Fix the guard"), "Fix the guard\n"),
        ("whitespace only", &one, Some(" \n\t "), "land: 1 requests\n\n- r1: Add squash ref\n"),
    ];
    for (name, requests, caller, expected) in rows {
        assert_eq!(squash_message("land", requests, caller), expected, "{name}");
    }
}
