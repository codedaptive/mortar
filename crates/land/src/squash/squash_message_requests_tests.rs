//! squash_message_requests: messages written out as squash_message writes them, with a trailer paragraph appended
//! as text. 0 requests gives an empty list; one and several are read in message order; a count larger or smaller
//! than the request lines, a subject not of the form and a "- " line without ": " are None; a request line whose
//! subjects hold ": " and "; " is read to its first ": " only.

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
    let message = with_trailers("land: 1 requests\n\n- WO-land-5: add squash\n");
    assert_eq!(squash_message_requests(&message), Some(strings(&["WO-land-5"])));
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
fn a_subject_not_of_the_form_is_none() {
    for subject in ["land: two requests\n", "land 1 requests\n", "land: 1 request\n", "fix the build\n"] {
        let message = with_trailers(&format!("{subject}\n- r1: first\n"));
        assert_eq!(squash_message_requests(&message), None, "{subject:?}");
    }
}

#[test]
fn a_request_line_without_a_colon_space_is_none() {
    let message = with_trailers("land: 1 requests\n\n- r1 first\n");
    assert_eq!(squash_message_requests(&message), None);
}

#[test]
fn a_request_line_is_read_to_its_first_colon_space_only() {
    let message = with_trailers("land: 1 requests\n\n- WO-land-7: fix: the guard; docs: a line\n");
    assert_eq!(squash_message_requests(&message), Some(strings(&["WO-land-7"])));
}
