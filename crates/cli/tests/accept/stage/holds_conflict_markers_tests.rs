use super::holds_conflict_markers;

#[test]
fn a_written_conflict_holds_markers() {
    let text = "<<<<<<< ours\na from r1\n=======\na from r2\n>>>>>>> theirs\n";
    assert!(holds_conflict_markers(text));
}

#[test]
fn each_marker_alone_counts() {
    assert!(holds_conflict_markers("x\n<<<<<<< HEAD\n"));
    assert!(holds_conflict_markers("x\n=======\n"));
    assert!(holds_conflict_markers(">>>>>>> 0a1b\n"));
    assert!(holds_conflict_markers("||||||| base\n"));
}

#[test]
fn plain_text_holds_none() {
    assert!(!holds_conflict_markers("a from r2\n"));
    assert!(!holds_conflict_markers(""));
}

#[test]
fn a_longer_rule_or_a_marker_inside_a_line_is_not_a_marker() {
    assert!(!holds_conflict_markers("========\n"));
    assert!(!holds_conflict_markers("text <<<<<<< inside\n"));
}
