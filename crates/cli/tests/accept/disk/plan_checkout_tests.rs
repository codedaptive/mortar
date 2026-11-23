use super::{plan_checkout, BuildCommand, CheckoutPlan, TreeFile, ZonePlan};

fn file(mode: &str, path: &str, size: Option<u64>) -> TreeFile {
    TreeFile { mode: mode.to_string(), path: path.to_string(), size }
}

fn zone(zone_dir: &str, request_dir: &str, edit_file: &str, build: Result<BuildCommand, String>) -> ZonePlan {
    ZonePlan {
        zone_dir: zone_dir.to_string(),
        request_dir: request_dir.to_string(),
        edit_file: edit_file.to_string(),
        build,
    }
}

fn cargo(dir: &str) -> Result<BuildCommand, String> {
    Ok(BuildCommand { dir: dir.to_string(), program: "cargo".to_string(), args: vec!["build".to_string()] })
}

fn strings(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

const NO_TOP_BUILD: &str = "the copy's top level holds no Cargo.toml or Package.swift";
const NO_KIT: &str =
    "HEAD holds no package folder (a folder below the top level with its own Cargo.toml or Package.swift and a file in a subfolder)";

#[test]
fn plan_checkout_plans_the_heaviest_zone_the_kit_and_the_root_files() {
    let rows: Vec<(&str, Vec<TreeFile>, Result<CheckoutPlan, String>)> = vec![
        (
            "all sizes summed, the heaviest zone, its first deep regular file; no build and no kit",
            vec![
                file("100644", "root one", Some(8)),
                file("160000", "root-module", None),
                file("100644", "beta/top", Some(7)),
                file("120000", "beta/sub/link", Some(5)),
                file("100755", "beta/sub/deep/run", Some(11)),
                file("100644", "beta/other/later", Some(2)),
                file("100644", "alpha/sub/file", Some(20)),
                file("100644", "root-two", Some(3)),
            ],
            Ok(CheckoutPlan {
                bytes: 56,
                zone: zone("beta", "beta/sub/deep", "beta/sub/deep/run", Err(NO_TOP_BUILD.to_string())),
                kit: Err(NO_KIT.to_string()),
                root_files: strings(&["root one", "root-two"]),
            }),
        ),
        (
            "equal zone totals choose the name that sorts first, not input order",
            vec![file("100644", "z/sub/z", Some(10)), file("100644", "a/sub/a", Some(10))],
            Ok(CheckoutPlan {
                bytes: 20,
                zone: zone("a", "a/sub", "a/sub/a", Err(NO_TOP_BUILD.to_string())),
                kit: Err(NO_KIT.to_string()),
                root_files: vec![],
            }),
        ),
        (
            "the kit is the lightest package, built in its own folder; the zone is built at the top",
            vec![
                file("100644", "Cargo.toml", Some(50)),
                file("100644", "crates/big/Cargo.toml", Some(10)),
                file("100644", "crates/big/src/lib.rs", Some(500)),
                file("100644", "crates/small/Cargo.toml", Some(10)),
                file("100644", "crates/small/src/lib.rs", Some(30)),
                file("100644", "docs/guide.md", Some(5)),
            ],
            Ok(CheckoutPlan {
                bytes: 605,
                zone: zone("crates", "crates/big", "crates/big/Cargo.toml", cargo("")),
                kit: Ok(zone("crates/small", "crates/small/src", "crates/small/src/lib.rs", cargo("crates/small"))),
                root_files: strings(&["Cargo.toml"]),
            }),
        ),
        (
            "a kit not found leaves the plan whole",
            vec![file("100644", "Cargo.toml", Some(1)), file("100644", "src/bin/main.rs", Some(9))],
            Ok(CheckoutPlan {
                bytes: 10,
                zone: zone("src", "src/bin", "src/bin/main.rs", cargo("")),
                kit: Err(NO_KIT.to_string()),
                root_files: strings(&["Cargo.toml"]),
            }),
        ),
        ("an empty tree", vec![], Err("HEAD holds no folder at its top level".to_string())),
        (
            "no top-level folder",
            vec![file("100644", "root", Some(1))],
            Err("HEAD holds no folder at its top level".to_string()),
        ),
    ];
    for (name, files, expected) in rows {
        assert_eq!(plan_checkout(&files), expected, "{name}");
    }
}
