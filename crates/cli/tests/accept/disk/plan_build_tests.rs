use super::{plan_build, BuildCommand, TreeFile};

type Planned = Result<BuildCommand, String>;

fn file(mode: &str, path: &str) -> TreeFile {
    TreeFile { mode: mode.to_string(), path: path.to_string(), size: Some(1) }
}

fn build(dir: &str, program: &str) -> Result<BuildCommand, String> {
    Ok(BuildCommand { dir: dir.to_string(), program: program.to_string(), args: vec!["build".to_string()] })
}

#[test]
fn plan_build_picks_cargo_then_swift_from_the_dir_itself() {
    let none_at_top = "the copy's top level holds no Cargo.toml or Package.swift";
    let rows: Vec<(&str, Vec<TreeFile>, &str, Planned)> = vec![
        ("cargo at the top", vec![file("100644", "Cargo.toml"), file("100644", "src/main.rs")], "", build("", "cargo")),
        ("swift in a nested dir", vec![file("100644", "kits/Q/Package.swift")], "kits/Q", build("kits/Q", "swift")),
        (
            "cargo wins over swift",
            vec![file("100644", "k/Package.swift"), file("100755", "k/Cargo.toml")],
            "k",
            build("k", "cargo"),
        ),
        ("a link is not a manifest", vec![file("120000", "k/Cargo.toml")], "k", Err("k holds no Cargo.toml or Package.swift".to_string())),
        ("a deeper manifest is not the dir's", vec![file("100644", "a/b/Cargo.toml")], "a", Err("a holds no Cargo.toml or Package.swift".to_string())),
        ("a manifest under a dir is not the top's", vec![file("100644", "a/Package.swift")], "", Err(none_at_top.to_string())),
        ("a longer name is not the manifest", vec![file("100644", "k/xCargo.toml")], "k", Err("k holds no Cargo.toml or Package.swift".to_string())),
    ];
    for (name, files, dir, expected) in rows {
        assert_eq!(plan_build(&files, dir), expected, "{name}");
    }
}
