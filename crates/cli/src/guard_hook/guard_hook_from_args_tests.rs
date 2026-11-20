//! guard_hook_from_args: the hook's words as a GuardHook, or the reason they cannot be read.

use std::cell::Cell;

use land::GuardHook;

use super::guard_hook_from_args;

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| word.to_string()).collect()
}

#[test]
fn reference_transaction_reads_its_state_without_the_push_urls() {
    let called = Cell::new(false);
    let hook = guard_hook_from_args(&words(&["reference-transaction", "prepared"]), || {
        called.set(true);
        None
    });
    assert_eq!(hook, Ok(GuardHook::ReferenceTransaction { state: "prepared".to_string() }));
    assert!(!called.get(), "push_urls is not read for reference-transaction");
}

#[test]
fn pre_push_reads_remote_url_and_the_push_urls() {
    let called = Cell::new(false);
    let hook = guard_hook_from_args(&words(&["pre-push", "origin", "git@host:team/repo.git"]), || {
        called.set(true);
        Some(vec!["git@host:team/repo.git".to_string(), "https://host/team/repo".to_string()])
    });
    assert_eq!(
        hook,
        Ok(GuardHook::PrePush {
            remote: "origin".to_string(),
            url: "git@host:team/repo.git".to_string(),
            push_urls: Some(vec!["git@host:team/repo.git".to_string(), "https://host/team/repo".to_string()]),
        })
    );
    assert!(called.get());
}

#[test]
fn pre_push_with_unreadable_push_urls_carries_none() {
    let hook = guard_hook_from_args(&words(&["pre-push", "mirror", "/srv/mirror.git"]), || None);
    assert_eq!(
        hook,
        Ok(GuardHook::PrePush {
            remote: "mirror".to_string(),
            url: "/srv/mirror.git".to_string(),
            push_urls: None,
        })
    );
}

#[test]
fn no_hook_name_is_refused_naming_both_hooks() {
    let reason = guard_hook_from_args(&[], || None).expect_err("no hook name");
    assert_eq!(reason, "guard-hook takes the hook's name: reference-transaction or pre-push");
}

#[test]
fn an_unknown_hook_is_refused_naming_it_and_the_two_hooks() {
    let reason = guard_hook_from_args(&words(&["post-commit"]), || None).expect_err("unknown hook");
    assert_eq!(reason, "unknown hook \"post-commit\": the hooks are reference-transaction and pre-push");
}

#[test]
fn reference_transaction_with_the_wrong_word_count_is_refused() {
    let none = guard_hook_from_args(&words(&["reference-transaction"]), || None).expect_err("no state");
    assert_eq!(none, "reference-transaction takes one word, the state, and was given 0");
    let two = guard_hook_from_args(&words(&["reference-transaction", "prepared", "x"]), || None).expect_err("two");
    assert_eq!(two, "reference-transaction takes one word, the state, and was given 2");
}

#[test]
fn pre_push_with_the_wrong_word_count_is_refused_without_reading_push_urls() {
    let called = Cell::new(false);
    let one = guard_hook_from_args(&words(&["pre-push", "origin"]), || {
        called.set(true);
        None
    })
    .expect_err("one word");
    assert_eq!(one, "pre-push takes two words, the remote and the url, and was given 1");
    let three = guard_hook_from_args(&words(&["pre-push", "origin", "u", "x"]), || None).expect_err("three");
    assert_eq!(three, "pre-push takes two words, the remote and the url, and was given 3");
    assert!(!called.get());
}
