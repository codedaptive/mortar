use crate::cap_mismatch::cap_mismatch;
use crate::refused_file::RefusedFile;
use std::path::PathBuf;

fn refused(path: &str) -> RefusedFile {
    RefusedFile { path: PathBuf::from(path), size: 200, cap: 100 }
}

#[test]
fn dropped_exactly_the_refused_paths_gives_none() {
    let dropped = [PathBuf::from("b.bin"), PathBuf::from("a.bin")];
    assert_eq!(cap_mismatch(&[refused("a.bin"), refused("b.bin")], &dropped), Vec::<PathBuf>::new());
}

#[test]
fn a_refused_path_not_dropped_is_returned() {
    let dropped = [PathBuf::from("a.bin")];
    assert_eq!(cap_mismatch(&[refused("a.bin"), refused("a*b.bin")], &dropped), vec![PathBuf::from("a*b.bin")]);
}

#[test]
fn a_dropped_path_that_was_not_refused_is_returned() {
    let dropped = [PathBuf::from("big"), PathBuf::from("big/f")];
    assert_eq!(cap_mismatch(&[refused("big")], &dropped), vec![PathBuf::from("big/f")]);
}

#[test]
fn refused_paths_missing_come_before_extra_dropped_paths() {
    let dropped = [PathBuf::from("x.bin"), PathBuf::from("a.bin"), PathBuf::from("w.bin")];
    let refused = [refused("c.bin"), refused("a.bin"), refused("b.bin")];
    let expected = vec![PathBuf::from("c.bin"), PathBuf::from("b.bin"), PathBuf::from("x.bin"), PathBuf::from("w.bin")];
    assert_eq!(cap_mismatch(&refused, &dropped), expected);
}
