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

#[test]
#[cfg(target_os = "macos")]
fn clone_build_dirs_refuses_ancestor_swapped_for_symlink() {
    use crate::support::{setup, write, Fixture};
    use folders::{clone_build_dirs, make_request_folder, make_zone_folder, BuildDirClone};
    use std::{fs, os::unix::fs::symlink};
    use testkit::TestRepo;

    for (side, swap_parent) in [("folder's", false), ("parent's", true)] {
        let f = Fixture::new();
        let outside = TestRepo::new().unwrap();
        let outside_dir = outside.path.join("outside");
        fs::create_dir(&outside_dir).unwrap();
        let zone_path = f.repo.path.join("zone-folder");
        let req_path = f.repo.path.join("request-folder");
        make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
        write(&zone_path, "build/a/app.bin", b"parent a");
        write(&zone_path, "build/b/app.bin", b"parent b");
        make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();

        assert_eq!(clone_build_dirs(&zone_path, &req_path, &["build/a".into()]),
            vec![BuildDirClone::Shared { dir: "build/a".into() }], "{side}");
        assert_eq!(fs::read(req_path.join("build/a/app.bin")).unwrap(), b"parent a", "{side}");

        let swapped = if swap_parent { &zone_path } else { &req_path };
        if swap_parent { write(&outside_dir, "b/app.bin", b"outside b"); }
        fs::rename(swapped.join("build"), swapped.join("build-real")).unwrap();
        symlink(&outside_dir, swapped.join("build")).unwrap();

        let outcomes = clone_build_dirs(&zone_path, &req_path, &["build/b".into()]);
        assert!(matches!(outcomes.as_slice(), [BuildDirClone::Refused { dir, why }]
            if dir == "build/b" && why.contains(&format!("the {side} \"build\"")) && why.contains("nothing copied")),
            "{side}: {outcomes:?}");
        if swap_parent {
            assert!(fs::symlink_metadata(req_path.join("build/b")).is_err(), "{side}");
        } else {
            assert_eq!(fs::read_dir(&outside_dir).unwrap().count(), 0, "{side}");
        }
    }
}

#[test]
#[cfg(target_os = "macos")]
fn clone_build_dirs_keeps_todays_edge_outcomes() {
    use crate::support::{setup, write, Fixture};
    use folders::{clone_build_dirs, make_request_folder, make_zone_folder, BuildDirClone};
    use std::fs;

    let mut failures = Vec::new();
    for (case, dir, obstacle, holder, error) in [
        ("file at build", "build/out", Some("build"), "build", Some("File exists (os error 17)")),
        ("file at a", "a/b/out", Some("a"), "a/b", Some("Not a directory (os error 20)")),
        ("missing folder root", "target", None, "", None),
    ] {
        let f = Fixture::new();
        let zone_path = f.repo.path.join("zone-folder");
        let req_path = f.repo.path.join("request-folder");
        make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
        write(&zone_path, &format!("{dir}/app.bin"), b"parent output");
        if let Some(obstacle) = obstacle {
            make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();
            write(&req_path, obstacle, b"file");
        }
        let expected = match error {
            Some(error) => BuildDirClone::Refused {
                dir: dir.into(),
                why: format!("could not make {}: {error}; nothing copied", req_path.join(holder).display()),
            },
            None => BuildDirClone::Shared { dir: dir.into() },
        };
        let actual = clone_build_dirs(&zone_path, &req_path, &[dir.into()]);
        if actual != [expected.clone()] {
            failures.push(format!("{case}: expected {expected:?}, got {actual:?}"));
        }
        if error.is_none() {
            match fs::read(req_path.join("target/app.bin")) {
                Ok(bytes) if bytes == b"parent output" => (),
                result => failures.push(format!("{case}: expected parent output, got {result:?}")),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[cfg(target_os = "macos")]
fn clone_build_dirs_keeps_todays_permission_outcomes() {
    use crate::support::{setup, write, Fixture};
    use folders::{clone_build_dirs, make_request_folder, make_zone_folder, BuildDirClone};
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

    struct RestoreMode(PathBuf, fs::Permissions);
    impl Drop for RestoreMode {
        fn drop(&mut self) {
            fs::set_permissions(&self.0, self.1.clone()).expect("restore directory permissions");
        }
    }

    let mut failures = Vec::new();
    for (case, dir, change_parent, mode, expected) in [
        ("parent can't be looked up", "build/out", true, 0o600,
            Some(BuildDirClone::Absent { dir: "build/out".into() })),
        ("seen but can't be opened", "target", true, 0o000,
            Some(BuildDirClone::Refused { dir: "target".into(), why: "clonefile refused: Permission denied (os error 13)".into() })),
        ("search-only folder ancestor", "build/out", false, 0o300,
            Some(BuildDirClone::Shared { dir: "build/out".into() })),
        ("search-only parent ancestor", "build/out", true, 0o100,
            Some(BuildDirClone::Shared { dir: "build/out".into() })),
        ("folder ancestor without search", "build/out", false, 0o600,
            Some(BuildDirClone::Refused { dir: "build/out".into(), why: "clonefile refused: Permission denied (os error 13)".into() })),
        ("deeper folder ancestor without search", "a/b/out", false, 0o600, None),
        ("unreadable file as the holder", "build/out", false, 0o000, None),
    ] {
        let f = Fixture::new();
        let zone_path = f.repo.path.join("zone-folder");
        let req_path = f.repo.path.join("request-folder");
        make_zone_folder(&setup(&f.git, &zone_path), "red", &["kit/zone".into()], Some(&f.head)).unwrap();
        write(&zone_path, &format!("{dir}/app.bin"), b"parent out");
        make_request_folder(&setup(&f.git, &req_path), "red", "one", &["kit/zone".into()]).unwrap();
        let component = if dir == "a/b/out" { "a" } else if dir == "target" { "target" } else { "build" };
        if case == "unreadable file as the holder" {
            write(&req_path, "build", b"file");
        } else if !change_parent {
            fs::create_dir(req_path.join(component)).unwrap();
        }
        let changed = if change_parent { zone_path.join(component) } else { req_path.join(component) };
        let expected = expected.unwrap_or_else(|| BuildDirClone::Refused {
            dir: dir.into(),
            why: if case == "unreadable file as the holder" {
                format!("could not make {}: File exists (os error 17); nothing copied", req_path.join("build").display())
            } else {
                format!("could not make {}: Permission denied (os error 13); nothing copied", req_path.join("a/b").display())
            },
        });
        let guard = RestoreMode(changed.clone(), fs::metadata(&changed).unwrap().permissions());
        fs::set_permissions(&changed, fs::Permissions::from_mode(mode)).unwrap();
        let actual = clone_build_dirs(&zone_path, &req_path, &[dir.into()]);
        drop(guard);
        if actual != [expected.clone()] {
            failures.push(format!("{case}: expected {expected:?}, got {actual:?}"));
        }
        if matches!(expected, BuildDirClone::Shared { .. }) {
            match fs::read(req_path.join(format!("{dir}/app.bin"))) {
                Ok(bytes) if bytes == b"parent out" => (),
                result => failures.push(format!("{case}: expected parent out, got {result:?}")),
            }
        }
        if case == "folder ancestor without search" && fs::symlink_metadata(req_path.join("build/out")).is_ok() {
            failures.push(format!("{case}: build/out exists"));
        }
        if case == "deeper folder ancestor without search" && fs::symlink_metadata(req_path.join("a/b")).is_ok() {
            failures.push(format!("{case}: a/b exists"));
        }
        if case == "unreadable file as the holder" && !fs::symlink_metadata(req_path.join("build")).is_ok_and(|m| m.is_file()) {
            failures.push(format!("{case}: build is not a regular file"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
