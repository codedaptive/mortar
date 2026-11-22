//! squash_message_requests: messages written out as squash_message writes them, with a trailer paragraph appended
//! as text. Mortar's subject: 0 requests gives an empty list; one and several are read in message order; a count
//! larger or smaller than the request lines and a "- " line without ": " are None; a request line whose subjects
//! hold ": " and "; " is read to its first ": " only. A caller's subject: the request paragraph is read with no
//! count, and a message with no all-"- " paragraph between the first and the trailers has no requests.

use super::squash_message_requests;

const TRAILERS: &str = "Mortar-Change-Id: klmnopqrstuvwxyzklmnopqrstuvwxyz\nMortar-Zone: land\n\
                        Mortar-Squash-Of: 4444444444444444444444444444444444444444\n";

fn with_trailers(text: &str) -> String {
    format!("{text}\n{TRAILERS}")
}

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn zero_requests_is_an_empty_list() {
    assert_eq!(squash_message_requests(&with_trailers("land: 0 requests\n")), Some(Vec::new()));
}

#[test]
fn one_request_is_read() {
    let message = with_trailers("land: 1 requests\n\n- r1: add squash\n");
    assert_eq!(squash_message_requests(&message), Some(strings(&["r1"])));
}

#[test]
fn several_requests_keep_message_order() {
    let message = with_trailers("land: 3 requests\n\n- r2: second\n- r1: first; again\n- r3: third\n");
    assert_eq!(squash_message_requests(&message), Some(strings(&["r2", "r1", "r3"])));
}

#[test]
fn a_count_larger_than_the_lines_is_none() {
    let message = with_trailers("land: 3 requests\n\n- r1: first\n- r2: second\n");
    assert_eq!(squash_message_requests(&message), None);
}

#[test]
fn a_count_smaller_than_the_lines_is_none() {
    let message = with_trailers("land: 1 requests\n\n- r1: first\n- r2: second\n");
    assert_eq!(squash_message_requests(&message), None);
}

#[test]
fn a_caller_subject_reads_its_request_paragraph_with_no_count() {
    let rows: [(&str, &str, &[&str]); 6] = [
        ("subject and requests", "fix the build\n\n- r1: first\n- r2: second\n", &["r1", "r2"]),
        (
            "subject, body and requests",
            "fix the build\n\nThe guard waits.\nIt reads the lock.\n\n- r2: second\n- r1: first\n",
            &["r2", "r1"],
        ),
        ("no requests", "fix the build\n\nThe guard waits.\n", &[]),
        ("body paragraph mixing - and other lines", "fix the build\n\n- r1: first\nand a line\n", &[]),
        ("a single line - a: b is the subject, never requests", "- a: b\n", &[]),
        ("land: 1 request is not Mortar's form, so a caller's subject", "land: 1 request\n\n- r1: first\n", &["r1"]),
    ];
    for (row, text, expected) in rows {
        assert_eq!(squash_message_requests(&with_trailers(text)), Some(strings(expected)), "{row}");
    }
}

#[test]
fn a_request_line_without_a_colon_space_is_none() {
    let message = with_trailers("land: 1 requests\n\n- r1 first\n");
    assert_eq!(squash_message_requests(&message), None);
}

#[test]
fn a_request_line_is_read_to_its_first_colon_space_only() {
    let message = with_trailers("land: 1 requests\n\n- r1: fix: the guard; docs: a line\n");
    assert_eq!(squash_message_requests(&message), Some(strings(&["r1"])));
}
