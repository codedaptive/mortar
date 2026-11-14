use super::support::{path_refused, Fixture};
use std::path::PathBuf;

#[test]
fn rit_a_an_overlap_outside_current_files_names_the_all_files_path() {
    // resolve -> continue_replay (current files succeeds) -> resolved_folder_tree -> typed refusal/Display.
    // S-109: a typed overlap with neither path in the MAPPED set is unreachable. The public final handoff
    // instead must map against all_files, even when both overlap paths are absent from the current round.
    let f = Fixture::new();
    f.state(r#"["src/a.rs"]"#, r#"["lib","lib/x","src/a.rs"]"#, true);
    let git = f.runner(b"", b"", None);
    let before = f.snapshot();
    assert_eq!(path_refused(f.resolve(&git).unwrap_err()), (
        Some(PathBuf::from("lib")),
        "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"".into(),
    ));
    // The first overlay and resolved commit ran for real; only the later all_files overlay refused.
    assert_eq!(f.trace(), "ls-tree\nread-tree\nupdate-index\nwrite-tree\ncommit-tree\n");
    f.unchanged(&before);
}

#[test]
fn rit_a_an_empty_unusable_path_is_refused_naming_it_through_resolve() {
    // resolve -> continue_replay -> Git::tree_with_paths_typed -> tree_refusal -> real refusal Display.
    let f = Fixture::new();
    f.state(r#"[""]"#, r#"[""]"#, true);
    let git = f.runner(b"", b"", None);
    let before = f.snapshot();
    assert_eq!(path_refused(f.resolve(&git).unwrap_err()), (
        Some(PathBuf::from("")), "not a usable path (empty): \"\"".into(),
    ));
    assert_eq!(f.trace(), "", "refuse before listing or building any tree");
    f.unchanged(&before);
}

#[test]
fn rit_b_an_overlap_names_the_first_of_its_paths_in_files_order_with_its_reason() {
    // resolve -> continue_replay -> gitio typed Overlap -> tree_refusal/Display, before any tree work.
    refused_before_tree_work(
        r#"["src/a.rs","my \"odd\" file.txt","lib","lib/x"]"#, "lib",
        "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"",
    );
}

#[test]
fn rit_b_an_overlap_takes_the_inner_path_when_it_comes_first_in_files() {
    // Same public boundary with reversed selection: the named path follows caller order, not shortest path.
    refused_before_tree_work(
        r#"["lib/x","lib"]"#, "lib/x",
        "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"",
    );
}

#[test]
fn rit_b_a_dotdot_path_not_as_stored_is_refused_naming_it_with_its_reason() {
    // resolve's durable reader -> real gitio NotAsStored/Display, not normalized to a different tracked path.
    refused_before_tree_work(
        r#"["ok.txt","a/../b"]"#, "a/../b",
        "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a/../b\"",
    );
}

#[test]
fn rit_b_a_trailing_slash_path_is_refused_naming_it_with_its_reason() {
    // resolve -> real gitio refuses a folder-children path rather than taking its entries.
    refused_before_tree_work(
        r#"["dir/"]"#, "dir/",
        "not a usable path (ends in '/', naming a folder's children): \"dir/\"",
    );
}

fn refused_before_tree_work(files: &str, path: &str, reason: &str) {
    let f = Fixture::new();
    f.state(files, files, true);
    let git = f.runner(b"", b"", None);
    let before = f.snapshot();
    assert_eq!(path_refused(f.resolve(&git).unwrap_err()), (Some(PathBuf::from(path)), reason.to_string()));
    assert_eq!(f.trace(), "", "refuse before listing/building a tree or attempting a ref move");
    f.unchanged(&before);
}
