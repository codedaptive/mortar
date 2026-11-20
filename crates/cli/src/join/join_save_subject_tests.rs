//! join_save_subject: the request's name, then ": work saved at join".

use super::join_save_subject;

#[test]
fn the_subject_names_the_request() {
    assert_eq!(join_save_subject("fix-1"), "fix-1: work saved at join");
}
