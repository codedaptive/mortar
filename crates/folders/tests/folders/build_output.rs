#[test]
#[cfg(target_os = "macos")]
fn clone_build_dirs_shares_output_and_keeps_request_changes() {
    use crate::support::{setup, write, Fixture};
    use folders::{clone_build_dirs, make_request_folder, make_zone_folder, BuildDirClone};
    use std::fs;

    let f = Fixture::new();
    let zone_path = f.repo.path.join("zone-folder");
    let req_path = f.repo.path.join("request-folder");
    make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
    write(&zone_path, "target/debug/app.bin", b"original app");
    write(&zone_path, "target/note.txt", b"original note");
    make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();

    assert_eq!(
        clone_build_dirs(&zone_path, &req_path, &["target".into(), "absent-build".into()]),
        vec![BuildDirClone::Shared { dir: "target".into() }, BuildDirClone::Absent { dir: "absent-build".into() }],
    );
    assert_eq!(fs::read(req_path.join("target/debug/app.bin")).unwrap(), b"original app");
    assert_eq!(fs::read(req_path.join("target/note.txt")).unwrap(), b"original note");
    write(&req_path, "target/debug/app.bin", b"request app");
    write(&zone_path, "target/note.txt", b"zone note");
    assert_eq!(fs::read(zone_path.join("target/debug/app.bin")).unwrap(), b"original app");
    assert_eq!(fs::read(req_path.join("target/note.txt")).unwrap(), b"original note");
    assert!(!req_path.join("absent-build").exists());
    assert_eq!(
        clone_build_dirs(&zone_path, &req_path, &["target".into()]),
        vec![BuildDirClone::Kept { dir: "target".into() }],
    );
    assert_eq!(fs::read(req_path.join("target/debug/app.bin")).unwrap(), b"request app");
}

#[test]
#[cfg(target_os = "macos")]
fn clone_build_dirs_refuses_request_ancestor_symlink() {
    use crate::support::{setup, write, Fixture};
    use folders::{clone_build_dirs, make_request_folder, make_zone_folder, BuildDirClone};
    use std::{fs, os::unix::fs::symlink};
    use testkit::TestRepo;

    let f = Fixture::new();
    let outside = TestRepo::new().unwrap();
    let outside_dir = outside.path.join("outside");
    fs::create_dir(&outside_dir).unwrap();
    let zone_path = f.repo.path.join("zone-folder");
    let req_path = f.repo.path.join("request-folder");
    make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
    write(&zone_path, "build/out/app.bin", b"parent output");
    make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();
    symlink(&outside_dir, req_path.join("build")).unwrap();

    let outcomes = clone_build_dirs(&zone_path, &req_path, &["build/out".into()]);
    assert!(matches!(outcomes.as_slice(), [BuildDirClone::Refused { dir, why }]
        if dir == "build/out" && why.contains("the folder's \"build\"") && why.contains("nothing copied")), "{outcomes:?}");
    assert_eq!(fs::read_dir(&outside_dir).unwrap().count(), 0);
}

#[test]
#[cfg(target_os = "macos")]
fn clone_build_dirs_refuses_deep_request_ancestor_symlink() {
    use crate::support::{setup, write, Fixture};
    use folders::{clone_build_dirs, make_request_folder, make_zone_folder, BuildDirClone};
    use std::{fs, os::unix::fs::symlink};
    use testkit::TestRepo;

    let f = Fixture::new();
    let outside = TestRepo::new().unwrap();
    let outside_dir = outside.path.join("outside");
    fs::create_dir(&outside_dir).unwrap();
    let zone_path = f.repo.path.join("zone-folder");
    let req_path = f.repo.path.join("request-folder");
    make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
    write(&zone_path, "a/b/out/app.bin", b"parent output");
    make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();
    fs::create_dir(req_path.join("a")).unwrap();
    symlink(&outside_dir, req_path.join("a/b")).unwrap();

    let outcomes = clone_build_dirs(&zone_path, &req_path, &["a/b/out".into()]);
    assert!(matches!(outcomes.as_slice(), [BuildDirClone::Refused { dir, why }]
        if dir == "a/b/out" && why.contains("the folder's \"a/b\"") && why.contains("nothing copied")), "{outcomes:?}");
    assert_eq!(fs::read_dir(&outside_dir).unwrap().count(), 0);
}

#[test]
#[cfg(target_os = "macos")]
fn clone_build_dirs_refuses_parent_ancestor_symlink() {
    use crate::support::{setup, write, Fixture};
    use folders::{clone_build_dirs, make_request_folder, make_zone_folder, BuildDirClone};
    use std::{fs, os::unix::fs::symlink};
    use testkit::TestRepo;

    let f = Fixture::new();
    let outside = TestRepo::new().unwrap();
    let outside_dir = outside.path.join("outside");
    write(&outside_dir, "out/app.bin", b"outside output");
    let zone_path = f.repo.path.join("zone-folder");
    let req_path = f.repo.path.join("request-folder");
    make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
    make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();
    symlink(&outside_dir, zone_path.join("build")).unwrap();

    let outcomes = clone_build_dirs(&zone_path, &req_path, &["build/out".into()]);
    assert!(matches!(outcomes.as_slice(), [BuildDirClone::Refused { dir, why }]
        if dir == "build/out" && why.contains("the parent's \"build\"") && why.contains("nothing copied")), "{outcomes:?}");
    assert!(fs::symlink_metadata(req_path.join("build/out")).is_err());
}
