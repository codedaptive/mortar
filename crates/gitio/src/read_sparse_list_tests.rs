use crate::read_sparse_list::read_sparse_list;
use crate::Output;
use std::os::unix::ffi::OsStrExt;

fn output(stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status: 0,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

fn read(stdout: &[u8]) -> Vec<Vec<u8>> {
    read_sparse_list(&output(stdout, b""))
        .unwrap()
        .iter()
        .map(|p| p.as_os_str().as_bytes().to_vec())
        .collect()
}

#[test]
fn captured_cone_list_reads_every_name_in_order() {
    let stdout = b"-dash\n\"back\\\\sl\"\n\"caf\\303\\251\"\n\"cr\\rx\"\nd1\n\"q\\\"uote\"\nsp ace\n\"ta\\tb\"\n";
    assert_eq!(
        read(stdout),
        vec![
            b"-dash".to_vec(),
            b"back\\sl".to_vec(),
            b"caf\xc3\xa9".to_vec(),
            b"cr\rx".to_vec(),
            b"d1".to_vec(),
            b"q\"uote".to_vec(),
            b"sp ace".to_vec(),
            b"ta\tb".to_vec(),
        ]
    );
}

#[test]
fn captured_raw_utf8_line_is_its_bytes() {
    assert_eq!(read(b"caf\xc3\xa9\n"), vec![b"caf\xc3\xa9".to_vec()]);
}

#[test]
fn empty_cone_is_no_paths() {
    assert!(read(b"").is_empty());
}

#[test]
fn captured_non_cone_warning_is_refused_naming_it() {
    let stderr = b"warning: unrecognized pattern: '/d1/x'\nwarning: disabling cone pattern matching\n";
    let err = read_sparse_list(&output(b"/d1/\n!/d1/x\nd2\n", stderr)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(
        err.stderr.contains("\"warning: unrecognized pattern: '/d1/x'\\nwarning: disabling cone pattern matching\\n\""),
        "{}",
        err.stderr
    );
}

#[test]
fn a_warning_with_empty_stdout_is_still_refused() {
    assert!(read_sparse_list(&output(b"", b"warning: x\n")).is_err());
}

#[test]
fn missing_final_line_break_is_refused_naming_stdout() {
    let err = read_sparse_list(&output(b"d1\nd2", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"d1\\nd2\""), "{}", err.stderr);
}

#[test]
fn an_empty_line_is_refused() {
    let err = read_sparse_list(&output(b"d1\n\nd2\n", b"")).unwrap_err();
    assert!(err.stderr.contains("empty line"), "{}", err.stderr);
}

#[test]
fn a_badly_quoted_line_is_refused_naming_stdout() {
    let err = read_sparse_list(&output(b"d1\n\"ta\\tb\n", b"")).unwrap_err();
    assert!(err.stderr.contains("\"d1\\n\\\"ta\\\\tb\\n\""), "{}", err.stderr);
}
