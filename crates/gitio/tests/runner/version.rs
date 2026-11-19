//! The shim tests fix `git version`'s output, so Git::version's run-parse-floor wiring is checked whatever the
//! installed git's release. Every assertion holds on any host whose git can init the test repository: git 2.28 or
//! newer (GitRepo::new calls testkit's TestRepo::new, which runs `init -b main --object-format=sha1`; 2.28.0
//! accepts it, 2.27.0 refuses `-b`).

use crate::support::GitRepo;
use gitio::{Git, GitError, RunOpts, Version};

/// Git::version run through a shim that prints exactly `line` when called as `-C <dir> version`, and for any
/// other call names the arguments on stderr and exits 97, so a wrong subcommand fails the test.
fn shim_version(repo: &GitRepo, line: &str) -> Result<Version, GitError> {
    let shim = repo.path.join(".git/version-shim");
    let script = format!(
        "#!/bin/sh\n\
         if [ \"$#\" -eq 3 ] && [ \"$1\" = -C ] && [ \"$3\" = version ]; then printf '%s\\n' '{line}'; exit 0; fi\n\
         printf 'shim: not a git version call: %s\\n' \"$*\" >&2; exit 97\n"
    );
    testkit::write_git_shim(&shim, script.as_bytes()).unwrap();
    Git { program: shim, ..repo.git.clone() }.version()
}

/// The release `git version` output names, read by the rule Version::parse documents: the first word after
/// "git version " on the first line, as X.Y[.Z], a missing patch read as 0 and a fourth dotted part ignored.
fn release_token(stdout: &str) -> String {
    let first = stdout.lines().next().unwrap_or("");
    let rest = first.strip_prefix("git version ").unwrap_or_else(|| panic!("not git version output: {stdout:?}"));
    let word = rest.split_whitespace().next().unwrap_or_else(|| panic!("no release in: {stdout:?}"));
    let parts: Vec<&str> = word.split('.').collect();
    assert!(parts.len() >= 2, "release needs major and minor: {stdout:?}");
    format!("{}.{}.{}", parts[0], parts[1], parts.get(2).copied().unwrap_or("0"))
}

fn installed_version_stdout(repo: &GitRepo) -> String {
    let raw = repo.git.run(&["version"], &RunOpts {
        read_only: true,
        ..RunOpts::default()
    }).unwrap();
    String::from_utf8(raw.stdout).unwrap()
}

#[test]
fn git_version_refuses_git_2_45_4_with_the_floor_message() {
    assert_eq!(
        shim_version(&GitRepo::new(), "git version 2.45.4"),
        Err(GitError {
            args: Vec::new(),
            code: None,
            stderr: "git 2.45.4 found; mortar needs git 2.46 or newer".to_string(),
        })
    );
}

#[test]
fn git_version_returns_2_46_0_for_git_2_46_0() {
    assert_eq!(
        shim_version(&GitRepo::new(), "git version 2.46.0"),
        Ok(Version { major: 2, minor: 46, patch: 0 })
    );
}

#[test]
fn git_version_returns_2_46_0_for_apple_git_2_46_0() {
    assert_eq!(
        shim_version(&GitRepo::new(), "git version 2.46.0 (Apple Git-150)"),
        Ok(Version { major: 2, minor: 46, patch: 0 })
    );
}

#[test]
fn version_parse_of_the_installed_gits_output_equals_its_release_token() {
    let stdout = installed_version_stdout(&GitRepo::new());
    let v = Version::parse(stdout.as_bytes()).unwrap();
    assert_eq!(format!("{}.{}.{}", v.major, v.minor, v.patch), release_token(&stdout));
}

#[test]
fn git_version_on_the_installed_git_returns_its_release_or_the_floor_refusal() {
    let repo = GitRepo::new();
    let token = release_token(&installed_version_stdout(&repo));
    match repo.git.version() {
        Ok(v) => assert_eq!(format!("{}.{}.{}", v.major, v.minor, v.patch), token),
        Err(e) => {
            assert_eq!(e.code, None);
            assert!(e.stderr.starts_with(&format!("git {token} found; ")), "{e:?}");
        }
    }
}
