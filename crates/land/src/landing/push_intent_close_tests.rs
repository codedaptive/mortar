//! push_intent_close: Done whenever the push landed, whatever the remote holds; Failed naming the ref and what the
//! remote holds otherwise (at the landing's start, elsewhere, or absent). Oids and IntentClose are built as data.

use super::push_intent_close;
use gitio::Oid;
use oplog::IntentClose;

fn oid(digit: char) -> Oid {
    Oid::parse(&digit.to_string().repeat(40)).unwrap()
}

#[test]
fn landed_at_new_is_done() {
    assert_eq!(push_intent_close(Some(&oid('2')), true, "refs/heads/main"), IntentClose::Done);
}

#[test]
fn landed_with_the_remote_past_new_is_done() {
    assert_eq!(push_intent_close(Some(&oid('4')), true, "refs/heads/main"), IntentClose::Done);
}

#[test]
fn not_landed_at_expected_is_failed_naming_the_ref_at_expected() {
    assert_eq!(
        push_intent_close(Some(&oid('1')), false, "refs/heads/main"),
        IntentClose::Failed { refs: vec![("refs/heads/main".to_string(), Some(oid('1')))] }
    );
}

#[test]
fn not_landed_elsewhere_is_failed_naming_where_the_remote_is() {
    assert_eq!(
        push_intent_close(Some(&oid('3')), false, "refs/heads/trunk"),
        IntentClose::Failed { refs: vec![("refs/heads/trunk".to_string(), Some(oid('3')))] }
    );
}

#[test]
fn not_landed_without_the_branch_is_failed_naming_it_absent() {
    assert_eq!(
        push_intent_close(None, false, "refs/heads/main"),
        IntentClose::Failed { refs: vec![("refs/heads/main".to_string(), None)] }
    );
}
