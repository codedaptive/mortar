use super::{plan_zone, BuildCommand, TreeFile, ZonePlan};

type Planned = Result<ZonePlan, String>;

fn file(mode: &str, path: &str) -> TreeFile {
    TreeFile { mode: mode.to_string(), path: path.to_string(), size: Some(1) }
}

#[test]
fn plan_zone_finds_the_first_deep_regular_file_and_the_build() {
    let kit = "packages/kits/QueueKit";
    let kit_files = vec![
        file("100644", "packages/kits/QueueKitX/Sources/b.swift"),
        file("100644", "packages/kits/QueueKit/Package.swift"),
        file("120000", "packages/kits/QueueKit/Sources/link"),
        file("100755", "packages/kits/QueueKit/Sources/Queue/a.swift"),
        file("100644", "packages/kits/QueueKit/Tests/t.swift"),
    ];
    let rows: Vec<(&str, Vec<TreeFile>, &str, &str, Planned)> = vec![
        (
            "a nested zone built in its own folder",
            kit_files,
            kit,
            kit,
            Ok(ZonePlan {
                zone_dir: kit.to_string(),
                request_dir: "packages/kits/QueueKit/Sources/Queue".to_string(),
                edit_file: "packages/kits/QueueKit/Sources/Queue/a.swift".to_string(),
                build: Ok(BuildCommand {
                    dir: kit.to_string(),
                    program: "swift".to_string(),
                    args: vec!["build".to_string()],
                }),
            }),
        ),
        (
            "a build that cannot be planned leaves the zone planned",
            vec![file("100644", "a/top"), file("100644", "a/sub/file")],
            "a",
            "",
            Ok(ZonePlan {
                zone_dir: "a".to_string(),
                request_dir: "a/sub".to_string(),
                edit_file: "a/sub/file".to_string(),
                build: Err("the copy's top level holds no Cargo.toml or Package.swift".to_string()),
            }),
        ),
        (
            "no regular file in a subfolder",
            vec![file("100644", "zone/top"), file("120000", "zone/sub/link"), file("160000", "zone/sub/module")],
            "zone",
            "",
            Err("zone holds no regular file in a subfolder".to_string()),
        ),
    ];
    for (name, files, zone_dir, build_dir, expected) in rows {
        assert_eq!(plan_zone(&files, zone_dir, build_dir), expected, "{name}");
    }
}
