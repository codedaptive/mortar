use super::mr_verb;

fn argv(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

#[test]
fn the_verb_is_the_first_argument_after_the_program() {
    assert_eq!(mr_verb(&argv(&["mr", "save", "r1: work"])), Some("save"));
    assert_eq!(mr_verb(&argv(&["mortar", "guard-hook", "reference-transaction"])), Some("guard-hook"));
}

#[test]
fn options_before_the_verb_are_skipped() {
    assert_eq!(mr_verb(&argv(&["mr", "--json", "status"])), Some("status"));
}

#[test]
fn a_call_with_only_options_or_nothing_names_no_verb() {
    assert_eq!(mr_verb(&argv(&["mr", "--help"])), None);
    assert_eq!(mr_verb(&argv(&["mr"])), None);
}
