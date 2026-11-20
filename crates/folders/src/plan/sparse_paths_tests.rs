//! The cone set: union, trailing '/' removed, sorted, deduplicated; and each refusal, naming its path where there is one.

use super::sparse_paths;

fn list(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| p.to_string()).collect()
}

fn refused(paths: &[&str]) -> String {
    match sparse_paths(&list(paths), &[]) {
        Err(err) => err.reason,
        Ok(cone) => panic!("{paths:?} gave {cone:?}"),
    }
}

#[test]
fn unions_sorts_and_dedupes() {
    let cone = sparse_paths(&list(&["crates/folders", "Makefile", "crates/gitio/"]), &list(&["crates/gitio", "a/b"]));
    assert_eq!(cone.unwrap(), list(&["Makefile", "a/b", "crates/folders", "crates/gitio"]));
}

#[test]
fn either_list_alone_is_enough() {
    assert_eq!(sparse_paths(&[], &list(&["docs/"])).unwrap(), list(&["docs"]));
    assert_eq!(sparse_paths(&list(&["src"]), &[]).unwrap(), list(&["src"]));
}

#[test]
fn removes_every_trailing_slash() {
    assert_eq!(sparse_paths(&list(&["a/b//"]), &[]).unwrap(), list(&["a/b"]));
}

#[test]
fn same_path_in_both_lists_appears_once() {
    assert_eq!(sparse_paths(&list(&["x", "x/"]), &list(&["x"])).unwrap(), list(&["x"]));
}

#[test]
fn refuses_an_empty_union() {
    assert_eq!(
        sparse_paths(&[], &[]).unwrap_err().reason,
        "no sparse paths given: a folder needs at least one"
    );
}

#[test]
fn refuses_an_empty_path() {
    assert_eq!(refused(&["ok", ""]), "sparse path \"\" is empty");
}

#[test]
fn refuses_an_absolute_path() {
    assert_eq!(refused(&["/etc"]), "sparse path \"/etc\" is absolute");
    assert_eq!(refused(&["/"]), "sparse path \"/\" is absolute");
}

#[test]
fn refuses_nul() {
    assert_eq!(refused(&["a\0b"]), "sparse path \"a\\0b\" holds NUL");
}

#[test]
fn refuses_dot_dotdot_and_empty_components() {
    for path in [".", "..", "a/./b", "a/../b", "../a", "a/..", "a//b"] {
        assert_eq!(
            refused(&[path]),
            format!("sparse path {path:?} has a \".\", \"..\" or empty component")
        );
    }
}

#[test]
fn a_refusal_in_the_needed_list_names_its_path() {
    let err = sparse_paths(&list(&["ok"]), &list(&["../up"])).unwrap_err();
    assert_eq!(err.reason, "sparse path \"../up\" has a \".\", \"..\" or empty component");
}

#[test]
fn dots_inside_a_name_are_kept() {
    assert_eq!(sparse_paths(&list(&[".github", "a..b/c.d"]), &[]).unwrap(), list(&[".github", "a..b/c.d"]));
}

// make_request_folder and make_zone_folder call sparse_paths before they take the folders lock or write
// anything, so this refusal is make refusing a line break before any write.
#[test]
fn refuses_a_line_break_in_either_list() {
    assert_eq!(refused(&["ok", "a\nb"]), "sparse path \"a\\nb\" holds a line break");
    let err = sparse_paths(&list(&["ok"]), &list(&["c\nd"])).unwrap_err();
    assert_eq!(err.reason, "sparse path \"c\\nd\" holds a line break");
}

#[test]
fn refuses_a_line_break_in_a_later_component() {
    assert_eq!(refused(&["crates/folders/x\n"]), "sparse path \"crates/folders/x\\n\" holds a line break");
}

#[test]
fn keeps_a_carriage_return() {
    assert_eq!(sparse_paths(&list(&["a/b\rc"]), &[]).unwrap(), list(&["a/b\rc"]));
}
