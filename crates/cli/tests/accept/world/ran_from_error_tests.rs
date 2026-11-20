use std::io;

use super::Ran;

#[test]
fn a_child_that_did_not_start_says_why() {
    let error = Ran::from_error("mr", io::Error::new(io::ErrorKind::NotFound, "no such file"));
    assert_eq!(error.seen(), "did not run / could not run mr: no such file");
}
