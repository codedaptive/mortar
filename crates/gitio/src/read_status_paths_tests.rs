use crate::read_status_paths::read_status_paths;
use crate::StatusPaths;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

const CAPTURED: &[u8] = b"1 .D N... 100644 100644 000000 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d d00491fd7e5bb6fa28c517a0bb32b8b506539d4d del\x00\
1 .M N... 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d d00491fd7e5bb6fa28c517a0bb32b8b506539d4d mod\x00\
1 A. N... 000000 100644 100644 0000000000000000000000000000000000000000 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d re named\x00\
1 D. N... 100644 000000 000000 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d 0000000000000000000000000000000000000000 ren\x00\
1 MM N... 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d 0cfbf08886fca9a91cb753ec8734c84fcbe52c9f stg\x00\
u UU N... 100644 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d ba2906d0666cf726c7eaadd2cd3db615dedfdf3a 2299c37978265a95cbe835a4b0f0bbf15aad5549 c\x00\
? n\nl\x00? ud/deep/u\x00? untr\x00";

fn paths(items: &[&[u8]]) -> Vec<PathBuf> {
    items.iter().map(|p| PathBuf::from(OsStr::from_bytes(p))).collect()
}

fn refusal(stdout: &[u8]) -> String {
    let err = read_status_paths(stdout).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
}

#[test]
fn captured_status_sorts_each_record_by_kind_in_gits_order() {
    assert_eq!(
        read_status_paths(CAPTURED),
        Ok(StatusPaths {
            changed: paths(&[b"del", b"mod", b"re named", b"ren", b"stg"]),
            untracked: paths(&[b"n\nl", b"ud/deep/u", b"untr"]),
            conflicted: paths(&[b"c"]),
        })
    );
}

#[test]
fn a_rename_record_gives_its_new_then_its_old_path() {
    let stdout = b"2 R. N... 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d d00491fd7e5bb6fa28c517a0bb32b8b506539d4d R100 re named\x00ren\x00? u\x00";
    assert_eq!(
        read_status_paths(stdout),
        Ok(StatusPaths {
            changed: paths(&[b"re named", b"ren"]),
            untracked: paths(&[b"u"]),
            conflicted: Vec::new(),
        })
    );
}

#[test]
fn a_path_holding_spaces_and_a_newline_is_kept_whole() {
    let stdout = b"1 .M N... 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d d00491fd7e5bb6fa28c517a0bb32b8b506539d4d a b\nc d\x00";
    assert_eq!(read_status_paths(stdout).unwrap().changed, paths(&[b"a b\nc d"]));
}

#[test]
fn a_non_utf8_path_is_kept_as_its_bytes() {
    assert_eq!(read_status_paths(b"? caf\xe9\x00").unwrap().untracked, paths(&[b"caf\xe9"]));
}

#[test]
fn a_clean_folder_is_three_empty_lists() {
    assert_eq!(read_status_paths(b""), Ok(StatusPaths::default()));
}

#[test]
fn output_without_its_final_nul_is_refused_naming_it() {
    assert_eq!(refusal(b"? untr"), "status output without its final NUL: \"? untr\"");
}

#[test]
fn an_ignored_record_is_refused_naming_it() {
    assert_eq!(refusal(b"! ign\x00"), "not a status record: \"! ign\"");
}

#[test]
fn a_header_record_is_refused_naming_it() {
    assert_eq!(refusal(b"# branch.oid abc\x00"), "not a status record: \"# branch.oid abc\"");
}

#[test]
fn an_empty_record_is_refused() {
    assert_eq!(refusal(b"? a\x00\x00? b\x00"), "not a status record: \"\"");
}

#[test]
fn a_lone_nul_is_refused() {
    assert_eq!(refusal(b"\x00"), "not a status record: \"\"");
}

#[test]
fn an_untracked_record_with_an_empty_path_is_refused() {
    assert_eq!(refusal(b"? \x00"), "not a status record: \"? \"");
}

#[test]
fn a_changed_record_with_too_few_fields_is_refused_naming_it() {
    assert_eq!(refusal(b"1 .M N... 100644 mod\x00"), "not a status record: \"1 .M N... 100644 mod\"");
}

#[test]
fn a_conflict_record_with_too_few_fields_is_refused() {
    let stdout = b"u UU N... 100644 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d c\x00";
    assert!(refusal(stdout).starts_with("not a status record: \"u UU"));
}

#[test]
fn a_rename_without_its_old_path_is_refused_naming_it() {
    let stdout = b"2 R. N... 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d d00491fd7e5bb6fa28c517a0bb32b8b506539d4d R100 new\x00";
    assert_eq!(
        refusal(stdout),
        "not a status record: \"2 R. N... 100644 100644 100644 d00491fd7e5bb6fa28c517a0bb32b8b506539d4d d00491fd7e5bb6fa28c517a0bb32b8b506539d4d R100 new\""
    );
}

#[test]
fn a_tag_without_its_space_is_refused() {
    assert_eq!(refusal(b"?untr\x00"), "not a status record: \"?untr\"");
}
