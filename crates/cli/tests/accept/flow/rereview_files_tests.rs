use super::rereview_files;

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

/// The range-diff `mr rereview` printed in a drive (2026-11-17, git's own layout; ids shortened) for zone a's squash
/// re-made after r4 edited line 2 of src/a/r1.txt: the message changed and src/a/r1.txt changed. The outer hunk
/// header `@@ src/a/a.txt` names the section the hunk starts in, and the ` ## src/a/r1.txt (new) ##` line inside
/// it moves to the edited file.
const EDITED: &str = "1:  dfb26d0 ! 1:  b0f9873 a: 2 requests
    @@ Metadata
     Author: Accept Drive <accept@example.invalid>

      ## Commit message ##
    -    a: 2 requests
    +    a: 3 requests

         - r1: r1: first; r1: work saved at join
         - r2: r2: second; r2: work saved at join
    +    - r4: r4: work saved at join

         Mortar-Change-Id: smzvspnsovtkzokoynokoqukylpsysnw
         Mortar-Zone: a
    -    Mortar-Squash-Of: f982cda9
    +    Mortar-Squash-Of: d3bece0a

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
fn only_the_edited_file_is_named() {
    assert_eq!(rereview_files(&lines(EDITED)), vec!["src/a/r1.txt".to_string()]);
}

#[test]
fn a_header_without_the_leading_space_also_starts_its_section() {
    let text = "    @@ Commit message\n    +## src/a/new.txt (new) ##\n    ++new";
    assert_eq!(rereview_files(&lines(text)), vec!["src/a/new.txt".to_string()]);
}

#[test]
fn a_file_header_line_inside_a_hunk_starts_its_section() {
    let text = "1:  0a1b ! 1:  4d5e a: 2 requests
    @@ Commit message

         a: 2 requests
    +## src/a/new.txt (new) ##
    ++new
     ## src/a/old.txt ##
    -+gone";
    assert_eq!(rereview_files(&lines(text)), vec!["src/a/new.txt".to_string(), "src/a/old.txt".to_string()]);
}

#[test]
fn a_context_suffix_on_the_hunk_header_is_dropped() {
    let text = "    @@ src/a/a.txt: fn main\n    ++x";
    assert_eq!(rereview_files(&lines(text)), vec!["src/a/a.txt".to_string()]);
}

#[test]
fn changes_in_the_message_or_metadata_only_name_no_file() {
    let text = "    @@ Metadata\n    -Author: x\n    +Author: y\n    @@ Commit message\n    -    old\n    +    new";
    assert!(rereview_files(&lines(text)).is_empty());
}

#[test]
fn nothing_changed_names_no_file() {
    let text = "nothing changed: 4d5e has the same tree and parents as 0a1b";
    assert!(rereview_files(&lines(text)).is_empty());
}
