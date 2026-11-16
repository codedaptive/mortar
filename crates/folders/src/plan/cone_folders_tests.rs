//! Top-level files dropped from a cone, every other path kept in order, and the first error returned at once.

use gitio::PathKind;

use super::cone_folders;

fn list(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| p.to_string()).collect()
}

/// The kinds of a fixed tree: Cargo.toml and README files, kit a folder, sub a gitlink, anything else absent.
/// Panics on a path holding '/', which cone_folders must never ask about.
fn kind(path: &str) -> Result<Option<PathKind>, String> {
    assert!(!path.contains('/'), "asked about {path}");
    Ok(match path {
        "Cargo.toml" | "README" => Some(PathKind::File),
        "kit" => Some(PathKind::Folder),
        "sub" => Some(PathKind::Gitlink),
        _ => None,
    })
}

#[test]
fn drops_a_top_level_file() {
    assert_eq!(cone_folders(&list(&["Cargo.toml", "kit"]), kind), Ok(list(&["kit"])));
}

#[test]
fn keeps_a_top_level_folder_gitlink_and_absent_path() {
    assert_eq!(cone_folders(&list(&["kit", "sub", "missing"]), kind), Ok(list(&["kit", "sub", "missing"])));
}

#[test]
fn keeps_every_path_holding_a_slash_without_asking() {
    assert_eq!(cone_folders(&list(&["kit/zone", "a/Cargo.toml"]), kind), Ok(list(&["kit/zone", "a/Cargo.toml"])));
}

#[test]
fn keeps_the_input_order() {
    let cone = list(&["zeta/x", "README", "sub", "kit/zone", "Cargo.toml", "alpha"]);
    assert_eq!(cone_folders(&cone, kind), Ok(list(&["zeta/x", "sub", "kit/zone", "alpha"])));
}

#[test]
fn only_top_level_files_give_an_empty_list() {
    assert_eq!(cone_folders(&list(&["Cargo.toml", "README"]), kind), Ok(Vec::new()));
}

#[test]
fn the_first_error_is_returned_and_nothing_more_is_asked() {
    let mut calls = 0;
    let got = cone_folders(&list(&["kit", "bad", "README", "sub"]), |path| {
        calls += 1;
        if path == "bad" { Err("broken".to_string()) } else { kind(path) }
    });
    assert_eq!(got, Err("broken".to_string()));
    assert_eq!(calls, 2);
}
