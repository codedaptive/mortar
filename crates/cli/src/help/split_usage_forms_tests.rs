//! split_usage_forms: one line per top-level alternative of the first "( a | b )" group, with the text before and
//! after it and single spaces; a bar inside brackets or a nested group does not split; no group is one line
//! (SPEC-cli 08, "A verb's help").
//! ZUT (rule 11): split_usage_forms is pure; every input is a literal.

use super::split_usage_forms;

#[test]
fn a_bar_inside_brackets_is_not_split() {
    assert_eq!(split_usage_forms("mr x (a [b | c] | d) [--json]"), vec!["mr x a [b | c] [--json]", "mr x d [--json]"]);
}

#[test]
fn a_nested_group_is_not_split() {
    assert_eq!(split_usage_forms("mr x (a (b | c) | d)"), vec!["mr x a (b | c)", "mr x d"]);
}

#[test]
fn a_usage_with_no_group_is_one_line_unchanged() {
    assert_eq!(split_usage_forms("mr x [--y a|b] [--json]"), vec!["mr x [--y a|b] [--json]"]);
}

#[test]
fn nothing_after_the_group_adds_no_space() {
    assert_eq!(split_usage_forms("mr x (a | b)"), vec!["mr x a", "mr x b"]);
}
