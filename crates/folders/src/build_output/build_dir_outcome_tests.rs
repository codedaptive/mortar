//! build_dir_outcome: each check in its order, and the clone runs only when every check passes.

use std::cell::Cell;

use super::{build_dir_outcome, BuildDirClone};

struct Row {
    name: &'static str,
    dir: &'static str,
    in_parent: bool,
    parent_is_symlink: bool,
    in_folder: bool,
    clone_gives: Result<(), &'static str>,
    want: BuildDirClone,
    want_cloned: bool,
}

fn refused(dir: &str, why: &str) -> BuildDirClone {
    BuildDirClone::Refused { dir: dir.to_string(), why: why.to_string() }
}

#[test]
fn build_dir_outcome_follows_its_checks_in_order() {
    let rows = [
        Row {
            name: "a dir climbing out of the folder is refused",
            dir: "../x",
            in_parent: true,
            parent_is_symlink: false,
            in_folder: false,
            clone_gives: Ok(()),
            want: refused("../x", r#"build dir "../x" is not a relative path inside the folder"#),
            want_cloned: false,
        },
        Row {
            name: "an absolute dir is refused",
            dir: "/abs",
            in_parent: true,
            parent_is_symlink: false,
            in_folder: false,
            clone_gives: Ok(()),
            want: refused("/abs", r#"build dir "/abs" is not a relative path inside the folder"#),
            want_cloned: false,
        },
        Row {
            name: "the folder itself is refused",
            dir: ".",
            in_parent: true,
            parent_is_symlink: false,
            in_folder: false,
            clone_gives: Ok(()),
            want: refused(".", r#"build dir "." is not a relative path inside the folder"#),
            want_cloned: false,
        },
        Row {
            name: "an empty dir is refused",
            dir: "",
            in_parent: true,
            parent_is_symlink: false,
            in_folder: false,
            clone_gives: Ok(()),
            want: refused("", r#"build dir "" is not a relative path inside the folder"#),
            want_cloned: false,
        },
        Row {
            name: "a dir the parent lacks is absent",
            dir: "target",
            in_parent: false,
            parent_is_symlink: false,
            in_folder: false,
            clone_gives: Ok(()),
            want: BuildDirClone::Absent { dir: "target".to_string() },
            want_cloned: false,
        },
        Row {
            name: "a dir the parent lacks is absent even when the folder has it",
            dir: "target",
            in_parent: false,
            parent_is_symlink: false,
            in_folder: true,
            clone_gives: Ok(()),
            want: BuildDirClone::Absent { dir: "target".to_string() },
            want_cloned: false,
        },
        Row {
            name: "a parent dir that is a symlink is refused",
            dir: "target",
            in_parent: false,
            parent_is_symlink: true,
            in_folder: false,
            clone_gives: Ok(()),
            want: refused("target", r#"the parent's build dir "target" is a symlink; a clone would point into the parent"#),
            want_cloned: false,
        },
        Row {
            name: "a dir the folder already has is kept",
            dir: "target",
            in_parent: true,
            parent_is_symlink: false,
            in_folder: true,
            clone_gives: Ok(()),
            want: BuildDirClone::Kept { dir: "target".to_string() },
            want_cloned: false,
        },
        Row {
            name: "a clone that works is shared",
            dir: "build/out",
            in_parent: true,
            parent_is_symlink: false,
            in_folder: false,
            clone_gives: Ok(()),
            want: BuildDirClone::Shared { dir: "build/out".to_string() },
            want_cloned: true,
        },
        Row {
            name: "a clone that fails is refused with its reason",
            dir: "target",
            in_parent: true,
            parent_is_symlink: false,
            in_folder: false,
            clone_gives: Err("clonefile refused: cross-device link"),
            want: refused("target", "clonefile refused: cross-device link"),
            want_cloned: true,
        },
    ];
    for row in rows {
        let cloned = Cell::new(false);
        let clone = row.in_parent.then_some(|| {
            cloned.set(true);
            row.clone_gives.map_err(str::to_string)
        });
        let got = build_dir_outcome(row.dir, row.parent_is_symlink, row.in_folder, clone);
        assert_eq!(got, row.want, "{}", row.name);
        assert_eq!(cloned.get(), row.want_cloned, "{}: clone called", row.name);
    }
}
