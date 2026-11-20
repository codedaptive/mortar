//! marked_lines: line numbers counted by hand in two-way and diff3 conflicts, CRLF, look-alikes outside a conflict,
//! markers with no space, longer markers, a clean file and bytes that are not UTF-8.

use crate::marked_lines::marked_lines;

#[test]
fn two_way_conflict_marks_its_three_marker_lines() {
    let text = b"fn a() {}\n<<<<<<< ours\nlet x = 1;\n=======\nlet x = 2;\n>>>>>>> theirs\nfn b() {}\n";
    assert_eq!(marked_lines(text), vec![2, 4, 6]);
}

#[test]
fn diff3_conflict_marks_the_base_marker_too() {
    let text = b"top\n<<<<<<< ours\none\n||||||| base\nzero\n=======\ntwo\n>>>>>>> theirs\nbottom";
    assert_eq!(marked_lines(text), vec![2, 4, 6, 8]);
}

#[test]
fn bare_markers_with_nothing_after_them_are_marked() {
    assert_eq!(marked_lines(b"<<<<<<<\na\n|||||||\nb\n=======\nc\n>>>>>>>\n"), vec![1, 3, 5, 7]);
}

#[test]
fn crlf_lines_are_marked_as_their_lf_forms() {
    let text = b"a\r\n<<<<<<< ours\r\nb\r\n=======\r\nc\r\n>>>>>>> theirs\r\n";
    assert_eq!(marked_lines(text), vec![2, 4, 6]);
}

#[test]
fn separator_and_base_lines_outside_a_conflict_are_not_marked() {
    let text = b"Title\n=======\n\n||||||| not a base\ntext\n<<<<<<< ours\nx\n=======\ny\n>>>>>>> theirs\n=======\n";
    assert_eq!(marked_lines(text), vec![6, 8, 10]);
}

#[test]
fn markers_without_a_space_are_not_marked() {
    let text = b"<<<<<<<x\n=======\n>>>>>>>y\n<<<<<<<<<<x\n\t>>>>>>> theirs\n";
    assert_eq!(marked_lines(text), Vec::<usize>::new());
}

#[test]
fn six_char_markers_and_padded_separators_are_not_marked() {
    let text = b"<<<<<< ours\n<<<<<<< ours\n======= x\n ======\n======\n>>>>>> theirs\n";
    assert_eq!(marked_lines(text), vec![2]);
}

#[test]
fn ten_char_markers_are_marked() {
    let text = b"<<<<<<<<<< ours\na\n||||||||||\nb\n==========\nc\n>>>>>>>>>> theirs\n";
    assert_eq!(marked_lines(text), vec![1, 3, 5, 7]);
}

#[test]
fn clean_file_has_no_marked_lines() {
    assert_eq!(marked_lines(b"fn main() {\n    println!(\"<<<<<<< not at the start\");\n}\n"), Vec::<usize>::new());
    assert_eq!(marked_lines(b""), Vec::<usize>::new());
}

#[test]
fn bytes_that_are_not_utf8_are_checked() {
    let text = b"\xff\xfe\n<<<<<<< ours\n\x80a\n=======\n\xc3\n>>>>>>> theirs \xff\n";
    assert_eq!(marked_lines(text), vec![2, 4, 6]);
}
