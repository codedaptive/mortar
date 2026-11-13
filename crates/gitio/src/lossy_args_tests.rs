use crate::lossy_args::lossy_args;
use std::ffi::OsString;

#[test]
fn utf8_args_are_kept_as_written() {
    let args = vec![OsString::from("commit"), OsString::from("-m"), OsString::from("two words; é")];
    assert_eq!(lossy_args(&args), vec!["commit", "-m", "two words; é"]);
}

#[test]
fn no_args_give_no_text() {
    assert!(lossy_args(&[]).is_empty());
}

#[cfg(unix)]
#[test]
fn a_non_utf8_arg_shows_the_replacement_character() {
    use std::os::unix::ffi::OsStringExt;
    let args = vec![OsString::from("show"), OsString::from_vec(vec![b'a', 0xff, b'b'])];
    assert_eq!(lossy_args(&args), vec!["show", "a\u{FFFD}b"]);
}
