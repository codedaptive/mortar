use std::io;

use super::Ran;

#[test]
#[ignore = "acceptance: make accept"]
fn a_child_that_did_not_start_says_why() {
    let error = Ran::from_error("mortar", io::Error::new(io::ErrorKind::NotFound, "no such file"));
    assert_eq!(error.seen(), "did not run / could not run mortar: no such file");
}
