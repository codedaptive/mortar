use std::fs;
use std::path::Path;

use super::{world_bin_dir, Scratch};

#[test]
#[ignore = "acceptance: make accept"]
fn the_bin_folder_links_both_names_to_the_built_binaries() {
    let scratch = Scratch::new().expect("scratch folder");
    let bin = world_bin_dir(scratch.path()).expect("bin folder");
    assert_eq!(bin, scratch.path().join("bin"));
    assert_eq!(fs::read_link(bin.join("mr")).expect("mr link"), Path::new(env!("CARGO_BIN_EXE_mr")));
    assert_eq!(fs::read_link(bin.join("mortar")).expect("mortar link"), Path::new(env!("CARGO_BIN_EXE_mortar")));
    let entries = fs::read_dir(&bin).expect("list bin");
    let mut names: Vec<String> =
        entries.map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    assert_eq!(names, ["mortar", "mr"]);
    scratch.close().expect("remove scratch");
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_second_bin_folder_in_the_same_root_is_an_error() {
    let scratch = Scratch::new().expect("scratch folder");
    world_bin_dir(scratch.path()).expect("first bin folder");
    assert_eq!(world_bin_dir(scratch.path()).map_err(|error| error.kind()), Err(std::io::ErrorKind::AlreadyExists));
    scratch.close().expect("remove scratch");
}
