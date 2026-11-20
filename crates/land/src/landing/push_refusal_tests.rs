//! push_refusal: no outcomes, only Pushed and UpToDate, and Refused among others (the first one wins).

use super::push_refusal;
use gitio::{PushOutcome, PushRefusal};

#[test]
fn no_outcomes_is_no_refusal() {
    assert_eq!(push_refusal(vec![]), None);
}

#[test]
fn pushed_and_up_to_date_are_no_refusal() {
    assert_eq!(push_refusal(vec![PushOutcome::Pushed, PushOutcome::UpToDate, PushOutcome::Pushed]), None);
}

#[test]
fn one_refused_among_others_is_its_refusal() {
    let outcomes = vec![PushOutcome::UpToDate, PushOutcome::Refused(PushRefusal::StaleLease), PushOutcome::Pushed];
    assert_eq!(push_refusal(outcomes), Some(PushRefusal::StaleLease));
}

#[test]
fn the_first_of_two_refusals_wins() {
    let outcomes = vec![
        PushOutcome::Pushed,
        PushOutcome::Refused(PushRefusal::Hook),
        PushOutcome::Refused(PushRefusal::Other("[rejected] (fetch first)".to_string())),
    ];
    assert_eq!(push_refusal(outcomes), Some(PushRefusal::Hook));
}
