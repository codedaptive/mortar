use crate::read_trailers::read_trailers;

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[test]
fn parse_output_is_pairs_in_order() {
    assert_eq!(
        read_trailers(b"Change-Id: abc\nSigned-off-by: A <a@x> folded more\nDoc-Level: minor\n").unwrap(),
        pairs(&[("Change-Id", "abc"), ("Signed-off-by", "A <a@x> folded more"), ("Doc-Level", "minor")])
    );
}

#[test]
fn empty_output_is_no_trailers() {
    assert_eq!(read_trailers(b"").unwrap(), pairs(&[]));
}

#[test]
fn value_splits_at_the_first_colon() {
    assert_eq!(read_trailers(b"See: http://x/y: z\n").unwrap(), pairs(&[("See", "http://x/y: z")]));
}

#[test]
fn only_one_leading_space_is_dropped() {
    assert_eq!(read_trailers(b"K:  two\n").unwrap(), pairs(&[("K", " two")]));
}

#[test]
fn value_without_a_space_is_kept() {
    assert_eq!(read_trailers(b"K:v\n").unwrap(), pairs(&[("K", "v")]));
}

#[test]
fn empty_value_is_kept() {
    assert_eq!(read_trailers(b"K:\n").unwrap(), pairs(&[("K", "")]));
}

#[test]
fn last_line_without_newline_is_read() {
    assert_eq!(read_trailers(b"K: v").unwrap(), pairs(&[("K", "v")]));
}

#[test]
fn line_without_colon_is_refused_naming_it() {
    let err = read_trailers(b"K: v\nno colon here\n").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"no colon here\""), "{}", err.stderr);
}

#[test]
fn non_utf8_is_refused() {
    assert!(read_trailers(b"K: caf\xe9\n").is_err());
}
