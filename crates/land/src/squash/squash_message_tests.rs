//! squash_message: the exact text for no request, one request and several.

use super::{squash_message, SquashRequest};

fn request(request: &str, subjects: &[&str]) -> SquashRequest {
    SquashRequest { request: request.to_string(), subjects: subjects.iter().map(|s| s.to_string()).collect() }
}

#[test]
fn no_requests_give_the_subject_line_alone() {
    assert_eq!(squash_message("land", &[]), "land: 0 requests\n");
}

#[test]
fn one_request_gives_the_subject_a_blank_line_and_its_line() {
    assert_eq!(
        squash_message("land", &[request("r1", &["Add squash ref", "Add squash op"])]),
        "land: 1 requests\n\n- r1: Add squash ref; Add squash op\n"
    );
}

#[test]
fn several_requests_give_one_line_each_in_order() {
    let requests = [request("r2", &["two a"]), request("r1", &["one a", "one b", "one c"]), request("r3", &["three"])];
    assert_eq!(
        squash_message("join", &requests),
        "join: 3 requests\n\n- r2: two a\n- r1: one a; one b; one c\n- r3: three\n"
    );
}

#[test]
fn every_request_line_starts_with_a_dash_and_a_space() {
    let message = squash_message("land", &[request("r1", &["one"]), request("r2", &["two"])]);
    let lines: Vec<&str> = message.lines().skip(2).collect();
    assert_eq!(lines, vec!["- r1: one", "- r2: two"]);
    assert!(lines.iter().all(|line| line.starts_with("- ")), "{message:?}");
}
