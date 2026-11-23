use super::{pick_kit, TreeFile};

fn file(mode: &str, path: &str, size: Option<u64>) -> TreeFile {
    TreeFile { mode: mode.to_string(), path: path.to_string(), size }
}

const NONE: &str =
    "HEAD holds no package folder (a folder below the top level with its own Cargo.toml or Package.swift and a file in a subfolder)";

#[test]
fn pick_kit_takes_the_lightest_package_folder_with_a_file_in_a_subfolder() {
    let rows: Vec<(&str, Vec<TreeFile>, Result<&str, &str>)> = vec![
        (
            "the lightest package",
            vec![
                file("100644", "crates/a/Cargo.toml", Some(10)),
                file("100644", "crates/a/src/lib.rs", Some(100)),
                file("100644", "crates/b/Cargo.toml", Some(5)),
                file("100644", "crates/b/src/lib.rs", Some(20)),
            ],
            Ok("crates/b"),
        ),
        (
            "a manifest at the top level makes no package folder",
            vec![file("100644", "Cargo.toml", Some(1)), file("100644", "src/main.rs", Some(1))],
            Err(NONE),
        ),
        (
            "a package with no file in a subfolder is passed over",
            vec![
                file("100644", "tiny/Package.swift", Some(1)),
                file("100644", "tiny/README", Some(1)),
                file("100644", "big/Package.swift", Some(5)),
                file("100644", "big/Sources/x.swift", Some(50)),
            ],
            Ok("big"),
        ),
        (
            "a tie goes to the name that sorts first",
            vec![
                file("100644", "b/Cargo.toml", Some(5)),
                file("100644", "b/src/x", Some(5)),
                file("100644", "a/Cargo.toml", Some(5)),
                file("100644", "a/src/x", Some(5)),
            ],
            Ok("a"),
        ),
        (
            "a submodule's size counts 0",
            vec![
                file("100644", "n/Cargo.toml", Some(1)),
                file("100644", "n/src/a", Some(1)),
                file("160000", "n/vendor/x", None),
                file("100644", "o/Cargo.toml", Some(1)),
                file("100644", "o/src/a", Some(2)),
            ],
            Ok("n"),
        ),
        (
            "a linked manifest and a linked deep file make no package",
            vec![
                file("120000", "l/Cargo.toml", Some(1)),
                file("100644", "l/src/x", Some(1)),
                file("100644", "p/Cargo.toml", Some(1)),
                file("120000", "p/sub/link", Some(1)),
            ],
            Err(NONE),
        ),
        (
            "a package inside a package is a package of its own",
            vec![
                file("100644", "outer/Cargo.toml", Some(1)),
                file("100644", "outer/inner/Cargo.toml", Some(1)),
                file("100644", "outer/inner/src/lib.rs", Some(1)),
            ],
            Ok("outer/inner"),
        ),
        ("an empty tree", vec![], Err(NONE)),
    ];
    for (name, files, expected) in rows {
        let expected = expected.map(str::to_string).map_err(str::to_string);
        assert_eq!(pick_kit(&files), expected, "{name}");
    }
}
