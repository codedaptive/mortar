//! path_warning_for: no line when the first `mr` on PATH is this build once both paths are canonical (the same path,
//! or another one carrying the Mortar marker, S-147); the other-mr line, naming that `mr` as PATH gave it, when it is
//! not or either path will not canonicalize; the no-mr line when there is none; nothing when this binary's path is
//! unknown. `canonical` is a fake over a map; `has_marker` a fake over a list of canonical paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::path_warning_for;

const THIS: &str = "/b/target/debug/mortar";

fn canonical_from(pairs: &[(&str, &str)]) -> impl Fn(&Path) -> Option<PathBuf> {
    let map: HashMap<PathBuf, PathBuf> =
        pairs.iter().map(|(from, to)| (PathBuf::from(from), PathBuf::from(to))).collect();
    move |path| map.get(path).cloned()
}

fn warning_with(
    this: Option<&str>,
    first: Option<&str>,
    pairs: &[(&str, &str)],
    marked: &[&str],
) -> Option<String> {
    let marked: Vec<PathBuf> = marked.iter().map(PathBuf::from).collect();
    let has_marker = move |path: &Path| marked.iter().any(|m| m == path);
    path_warning_for(this.map(Path::new), first.map(Path::new), &canonical_from(pairs), &has_marker)
}

fn warning_for(this: Option<&str>, first: Option<&str>, pairs: &[(&str, &str)]) -> Option<String> {
    warning_with(this, first, pairs, &[])
}

#[test]
fn a_copied_mortar_elsewhere_carrying_the_marker_gives_no_warning() {
    let pairs = [(THIS, THIS), ("/home/u/.cargo/bin/mr", "/home/u/.cargo/bin/mr")];
    assert_eq!(warning_with(Some(THIS), Some("/home/u/.cargo/bin/mr"), &pairs, &["/home/u/.cargo/bin/mr"]), None);
}

#[test]
fn the_marker_is_asked_of_the_canonical_candidate() {
    let pairs = [(THIS, THIS), ("/home/u/bin/mr", "/opt/mortar/bin/mortar")];
    assert_eq!(warning_with(Some(THIS), Some("/home/u/bin/mr"), &pairs, &["/opt/mortar/bin/mortar"]), None);
    assert_eq!(
        warning_with(Some(THIS), Some("/home/u/bin/mr"), &pairs, &["/home/u/bin/mr"]),
        Some(
            "the first mr on PATH is /home/u/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn a_myrepos_like_file_beside_mortar_warns() {
    let pairs = [(THIS, THIS), ("/b/target/debug/mr", "/b/target/debug/mr")];
    assert_eq!(
        warning_for(Some(THIS), Some("/b/target/debug/mr"), &pairs),
        Some(
            "the first mr on PATH is /b/target/debug/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn has_marker_false_warns() {
    let pairs = [(THIS, THIS), ("/opt/bin/mr", "/opt/bin/mr")];
    let never = |_: &Path| false;
    assert_eq!(
        path_warning_for(Some(Path::new(THIS)), Some(Path::new("/opt/bin/mr")), &canonical_from(&pairs), &never),
        Some(
            "the first mr on PATH is /opt/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn equal_canonical_paths_never_ask_has_marker() {
    let pairs = [(THIS, THIS), ("/opt/bin/mr", THIS)];
    let refuse = |path: &Path| -> bool { panic!("has_marker asked of {}", path.display()) };
    assert_eq!(
        path_warning_for(Some(Path::new(THIS)), Some(Path::new("/opt/bin/mr")), &canonical_from(&pairs), &refuse),
        None
    );
}

#[test]
fn a_foreign_mr_elsewhere_warns_naming_it() {
    let pairs = [(THIS, THIS), ("/usr/local/bin/mr", "/usr/local/bin/mr")];
    assert_eq!(
        warning_for(Some(THIS), Some("/usr/local/bin/mr"), &pairs),
        Some(
            "the first mr on PATH is /usr/local/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn a_foreign_mr_reached_through_a_link_is_named_as_path_gave_it() {
    let pairs = [(THIS, THIS), ("/home/u/bin/mr", "/opt/myrepos/mr")];
    assert_eq!(
        warning_for(Some(THIS), Some("/home/u/bin/mr"), &pairs),
        Some(
            "the first mr on PATH is /home/u/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn an_mr_that_will_not_canonicalize_warns() {
    let pairs = [(THIS, THIS)];
    assert_eq!(
        warning_for(Some(THIS), Some("/b/target/debug/mr"), &pairs),
        Some(
            "the first mr on PATH is /b/target/debug/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn this_binary_that_will_not_canonicalize_warns() {
    let pairs = [("/b/target/debug/mr", "/b/target/debug/mr")];
    assert_eq!(
        warning_for(Some(THIS), Some("/b/target/debug/mr"), &pairs),
        Some(
            "the first mr on PATH is /b/target/debug/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)"
                .to_string()
        )
    );
}

#[test]
fn no_mr_on_path_gives_the_no_mr_line() {
    let pairs = [(THIS, THIS)];
    assert_eq!(
        warning_for(Some(THIS), None, &pairs),
        Some("no mr is on PATH: run Mortar as mortar (/b/target/debug/mortar)".to_string())
    );
}

#[test]
fn no_path_for_this_binary_gives_no_warning() {
    let pairs = [("/usr/local/bin/mr", "/usr/local/bin/mr")];
    assert_eq!(warning_for(None, Some("/usr/local/bin/mr"), &pairs), None);
    assert_eq!(warning_for(None, None, &pairs), None);
}

#[test]
fn a_symlinked_mr_whose_canonical_path_is_this_binary_gives_no_warning() {
    let pairs = [(THIS, THIS), ("/usr/local/bin/mr", THIS)];
    assert_eq!(warning_for(Some(THIS), Some("/usr/local/bin/mr"), &pairs), None);
}
