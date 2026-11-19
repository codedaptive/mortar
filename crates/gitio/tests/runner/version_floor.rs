use crate::support::GitRepo;
use gitio::{RunOpts, Version};

#[test]
fn git_version_joins_real_output_to_parser_and_floor() {
    // Joins Git::run -> Git::version -> Version::parse -> Version::require_floor.
    let repo = GitRepo::new();
    let installed = repo.git.version().unwrap();
    let raw = repo.git.run(&["version"], &RunOpts {
        read_only: true,
        ..RunOpts::default()
    }).unwrap();
    let prefix = format!("git version {}.{}.{}", installed.major, installed.minor, installed.patch);
    assert!(String::from_utf8(raw.stdout).unwrap().starts_with(&prefix));
    assert!(installed >= Version { major: 2, minor: 46, patch: 0 });

    // The installed git cannot provide an old-version response: exercise that public
    // parse-to-refusal seam with the specification's boundary, not a fake executable.
    let old = Version::parse(b"git version 2.45.99\n").unwrap();
    let error = old.require_floor().unwrap_err();
    assert_eq!(error.code, None);
    assert!(error.stderr.contains("2.45.99"), "{error:?}");
    assert!(error.stderr.contains("2.46"), "{error:?}");
    let floor = Version::parse(b"git version 2.46.0\n").unwrap().require_floor().unwrap();
    assert_eq!(floor, Version { major: 2, minor: 46, patch: 0 });
}
