//! overlap_queries: one git log -p of `<base>..<tip>` over the whole tree per open request, in order.

use ::join::OpenRequest;
use gitio::{BaselineQuery, Oid};

use super::overlap_queries;

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333";
const D: &str = "4444444444444444444444444444444444444444";

fn request(name: &str, base: &str, tip: &str) -> OpenRequest {
    OpenRequest { name: name.to_string(), base: Oid::parse(base).unwrap(), tip: Oid::parse(tip).unwrap() }
}

fn log(range: &str) -> BaselineQuery {
    BaselineQuery::LogPatches { range: range.to_string(), paths: vec![] }
}

#[test]
fn no_requests_give_no_queries() {
    assert_eq!(overlap_queries(&[]), vec![]);
}

#[test]
fn one_request_gives_its_base_to_tip_range() {
    assert_eq!(
        overlap_queries(&[request("task", A, B)]),
        vec![log("1111111111111111111111111111111111111111..2222222222222222222222222222222222222222")]
    );
}

#[test]
fn two_requests_give_one_range_each_in_order() {
    assert_eq!(
        overlap_queries(&[request("task", A, B), request("peer", C, D)]),
        vec![
            log("1111111111111111111111111111111111111111..2222222222222222222222222222222222222222"),
            log("3333333333333333333333333333333333333333..4444444444444444444444444444444444444444"),
        ]
    );
}
