use super::rereview_only_edit;

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

/// The end of the range-diff `mr rereview` printed in a drive (2026-11-17, git's own layout): the message changed,
/// and in src/a/r1.txt line 2 went from `r1 line 2` to `r4 edit`.
const EDITED: &str = "1:  dfb26d0 ! 1:  b0f9873 a: 2 requests
    @@ Metadata
      ## Commit message ##
    -    a: 2 requests
    +    a: 3 requests

      ## src/a/a.txt ##
     @@
    @@ src/a/a.txt
      ## src/a/r1.txt (new) ##
     @@
     +r1 line 1
    -+r1 line 2
    ++r4 edit
     +r1 line 3";

#[test]
fn the_one_edit_and_nothing_else_in_the_file_holds() {
    assert!(rereview_only_edit(&lines(EDITED), "src/a/r1.txt", "r1 line 2", "r4 edit"));
}

#[test]
fn an_untouched_line_also_marked_changed_fails() {
    let text = EDITED.replace("     +r1 line 1", "    -+r1 line 1\n    ++r1 line one");
    assert!(!rereview_only_edit(&lines(&text), "src/a/r1.txt", "r1 line 2", "r4 edit"));
}

#[test]
fn another_edit_or_a_missing_half_fails() {
    assert!(!rereview_only_edit(&lines(EDITED), "src/a/r1.txt", "r1 line 2", "r4 other"));
    let added_only = EDITED.replace("    -+r1 line 2\n", "");
    assert!(!rereview_only_edit(&lines(&added_only), "src/a/r1.txt", "r1 line 2", "r4 edit"));
}

#[test]
fn the_edit_must_be_in_the_file_named() {
    assert!(!rereview_only_edit(&lines(EDITED), "src/a/a.txt", "r1 line 2", "r4 edit"));
}

#[test]
fn a_range_diff_with_nothing_changed_fails() {
    let text = "nothing changed: 4d5e has the same tree and parents as 0a1b";
    assert!(!rereview_only_edit(&lines(text), "src/a/r1.txt", "r1 line 2", "r4 edit"));
}
