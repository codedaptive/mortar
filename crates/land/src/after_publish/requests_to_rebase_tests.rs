//! requests_to_rebase: requests based elsewhere are listed in input order; those based on the new tip are left out.

use super::requests_to_rebase;
use gitio::Oid;
use join::OpenRequest;

const NEW: &str = "1111111111111111111111111111111111111111";
const OLD: &str = "2222222222222222222222222222222222222222";
const OTHER: &str = "3333333333333333333333333333333333333333";
const REQUEST_TIP: &str = "5555555555555555555555555555555555555555";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn request(name: &str, base: &str) -> OpenRequest {
    OpenRequest { name: name.to_string(), base: oid(base), tip: oid(REQUEST_TIP) }
}

#[test]
fn requests_off_the_new_tip_are_listed_in_order_and_those_on_it_left_out() {
    let requests = [request("c", OLD), request("a", NEW), request("b", OTHER), request("d", OLD)];
    let expected = vec![request("c", OLD), request("b", OTHER), request("d", OLD)];
    assert_eq!(requests_to_rebase(&requests, &oid(NEW)), expected);
}

#[test]
fn every_request_on_the_new_tip_leaves_nothing_to_rebase() {
    assert_eq!(requests_to_rebase(&[request("a", NEW), request("b", NEW)], &oid(NEW)), vec![]);
}

#[test]
fn no_requests_is_nothing_to_rebase() {
    assert_eq!(requests_to_rebase(&[], &oid(NEW)), vec![]);
}

#[test]
fn a_request_whose_tip_is_the_new_tip_but_based_elsewhere_is_listed() {
    let on_tip = OpenRequest { name: "a".to_string(), base: oid(OLD), tip: oid(NEW) };
    assert_eq!(requests_to_rebase(std::slice::from_ref(&on_tip), &oid(NEW)), vec![on_tip]);
}
