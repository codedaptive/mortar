//! CLI run_guard_install -> Land check_hooks_path/guard_install -> Gitio, then Git's installed
//! reference-transaction hook -> CLI run_guard_hook -> Land guard_decide from a different linked worktree.
//! The actual Cargo binary folder is first on PATH; the fixture owns every repository, worktree and hook.

use crate::json::check;
use crate::support::{raw, shim_path, Fixture};
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn git(fixture: &Fixture, cwd: &Path, args: &[&str]) -> Output {
    let binary = Path::new(env!("CARGO_BIN_EXE_mortar")).canonicalize().unwrap();
    Command::new("git")
        .arg("-C").arg(cwd).args(args)
        // In particular, no inherited MORTAR_OP may authorize the attempted protected move.
        .env_clear().envs(fixture.git.env.iter().cloned())
        .env("LC_ALL", "C").env("PATH", shim_path(binary.parent().unwrap()))
        .env("HOME", &fixture.root).env("TMPDIR", &fixture.root)
        .stdin(Stdio::null()).output().unwrap()
}

fn git_ok(fixture: &Fixture, cwd: &Path, args: &[&str]) -> String {
    let output = git(fixture, cwd, args);
    assert!(output.status.success(), "git {args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap().trim_end_matches('\n').to_string()
}

#[test]
fn absolute_hooks_installed_from_one_linked_worktree_guard_a_second_linked_worktree() {
    let fixture = Fixture::new();
    let old = fixture.base.as_str();
    let new = fixture.commit(&fixture.base, &[("src/file", b"second commit\n")], "second\n");
    assert_ne!(new, fixture.base);
    raw(&fixture.git, &["update-ref", "refs/heads/integration", old]);
    raw(&fixture.git, &["update-ref", "refs/heads/probe", old]);
    let hooks = fixture.root.join("shared hooks");
    assert!(hooks.is_absolute());
    raw(&fixture.git, &["config", "core.hooksPath", hooks.to_str().unwrap()]);

    let binary = Path::new(env!("CARGO_BIN_EXE_mortar")).canonicalize().unwrap();
    let installed = fixture.command(&fixture.zone, &["--actor", "harness", "--json", "guard", "install"])
        .env("PATH", shim_path(binary.parent().unwrap())).output().unwrap();
    let report = check(installed, "guard", 0, "installed");
    assert_eq!(report.text("hooks_dir"), hooks.to_str().unwrap());
    assert_eq!(std::fs::read_to_string(hooks.join("reference-transaction")).unwrap(),
        "#!/bin/sh\nexec mortar guard-hook reference-transaction \"$@\"\n");
    assert!(!fixture.common.join("hooks/reference-transaction").exists());

    // A worktree made after installation must be covered without installing a second copy of the hooks.
    let second = fixture.root.join("second linked worktree");
    git_ok(&fixture, &fixture.git.repo, &["worktree", "add", "-b", "second", second.to_str().unwrap()]);
    assert!(fixture.zone.join(".git").is_file());
    assert!(second.join(".git").is_file());
    assert_ne!(fixture.zone, second);
    assert_eq!(git_ok(&fixture, &second, &["rev-parse", "--path-format=absolute", "--git-common-dir"]),
        fixture.common.to_str().unwrap());
    assert_eq!(git_ok(&fixture, &second, &["config", "--type=path", "--get", "core.hooksPath"]),
        hooks.to_str().unwrap());
    assert_eq!(git_ok(&fixture, &second, &["rev-parse", "refs/heads/integration"]), old);

    // A real unprotected move succeeds through the same hook, excluding a missing/broken executable as the oracle.
    git_ok(&fixture, &second, &["update-ref", "refs/heads/probe", new.as_str(), old]);
    assert_eq!(git_ok(&fixture, &second, &["rev-parse", "refs/heads/probe"]), new.as_str());
    let refused = git(&fixture, &second, &["update-ref", "refs/heads/integration", new.as_str(), old]);
    assert!(!refused.status.success(), "unguarded second-worktree move: {refused:?}");
    assert_eq!(String::from_utf8(refused.stderr).unwrap(), concat!(
        "only Mortar moves refs/heads/integration\n",
        "owner: the landing surface: mr land and mr publish\n",
        "next: bring your work in with `mr join`, or ask for a landing\n",
        "fatal: in 'prepared' phase, update aborted by the reference-transaction hook\n",
    ));
    assert_eq!(git_ok(&fixture, &second, &["rev-parse", "refs/heads/integration"]), old);
    assert_eq!(git_ok(&fixture, &fixture.zone, &["rev-parse", "refs/heads/integration"]), old);
}
