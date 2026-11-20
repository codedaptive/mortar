use crate::read_remote_push_urls::read_remote_push_urls;

fn urls(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn one_line_is_one_url() {
    assert_eq!(read_remote_push_urls(b"https://h.example/r.git\n").unwrap(), urls(&["https://h.example/r.git"]));
}

#[test]
fn several_lines_keep_their_order() {
    let got = read_remote_push_urls(b"https://push1.example/o.git\nssh://push2.example/o.git\n").unwrap();
    assert_eq!(got, urls(&["https://push1.example/o.git", "ssh://push2.example/o.git"]));
}

#[test]
fn empty_output_is_no_urls() {
    assert_eq!(read_remote_push_urls(b"").unwrap(), Vec::<String>::new());
}

#[test]
fn a_cr_before_the_newline_stays_in_the_url() {
    assert_eq!(read_remote_push_urls(b"a\r\n").unwrap(), urls(&["a\r"]));
}

#[test]
fn a_line_with_spaces_is_kept_exactly() {
    assert_eq!(read_remote_push_urls(b"/tmp/has space/repo.git\n").unwrap(), urls(&["/tmp/has space/repo.git"]));
}

#[test]
fn missing_final_newline_is_refused_naming_the_text() {
    let err = read_remote_push_urls(b"a\nb").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "remote URLs without their final newline: \"a\\nb\"");
}

#[test]
fn non_utf8_is_refused_naming_the_lossy_text() {
    let err = read_remote_push_urls(b"/tmp/\xff\xfe.git\n").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "remote URLs that are not UTF-8: \"/tmp/\u{fffd}\u{fffd}.git\\n\"");
}
