use super::rereview_changes;

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

fn change(section: &str, line: &str) -> (String, String) {
    (section.to_string(), line.to_string())
}

#[test]
fn each_changed_line_comes_with_its_section_in_order() {
    let text = "1:  0a1b ! 1:  4d5e a: 2 requests
    @@ Metadata
      ## Commit message ##
    -    a: 2 requests
    +    a: 3 requests
    @@ src/a/a.txt
      ## src/a/r1.txt (new) ##
     +r1 line 1
    -+r1 line 2
    ++r4 edit";
    assert_eq!(
        rereview_changes(&lines(text)),
        vec![
            change("Commit message", "-    a: 2 requests"),
            change("Commit message", "+    a: 3 requests"),
            change("src/a/r1.txt", "-+r1 line 2"),
            change("src/a/r1.txt", "++r4 edit"),
        ]
    );
}

#[test]
fn a_changed_file_header_starts_its_own_section() {
    let text = "    @@ Commit message\n    +## src/a/new.txt (new) ##\n    ++new";
    assert_eq!(
        rereview_changes(&lines(text)),
        vec![change("src/a/new.txt", "+## src/a/new.txt (new) ##"), change("src/a/new.txt", "++new")]
    );
}

#[test]
fn context_pair_lines_and_a_change_before_any_section() {
    let text = "1:  0a1b ! 1:  4d5e subject\n     +context\n    +early";
    assert_eq!(rereview_changes(&lines(text)), vec![change("", "+early")]);
}
