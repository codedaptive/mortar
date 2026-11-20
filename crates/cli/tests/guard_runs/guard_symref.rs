//! Installed Land hooks -> Cargo's CLI run_guard_hook -> Land guard_decide and Gitio.
//! A disposable TestRepo owns the main and linked worktree; real Git supplies symbolic reference transactions.
//! The binary's own folder is first on PATH. No intent, prebuilt executable, fake hook or synthetic hook input.

use crate::support::shim_path;
use gitio::Git;
use land::LandRefs;
use std::{ffi::OsString, path::Path, process::{Command, Output, Stdio}};
use testkit::TestRepo;

fn env() -> Vec<(OsString, OsString)> {
    [
        ("GIT_CONFIG_GLOBAL", "/dev/null"), ("GIT_CONFIG_NOSYSTEM", "1"), ("LC_ALL", "C"),
        ("GIT_AUTHOR_NAME", "Guard Validation"), ("GIT_AUTHOR_EMAIL", "guard-validation@example.invalid"),
        ("GIT_COMMITTER_NAME", "Guard Validation"), ("GIT_COMMITTER_EMAIL", "guard-validation@example.invalid"),
    ].map(|(name, value)| (name.into(), value.into())).to_vec()
}

fn output(root: &Path, args: &[&str]) -> Output {
    let binary = Path::new(env!("CARGO_BIN_EXE_mortar")).canonicalize().unwrap();
    let bin = binary.parent().unwrap();
    assert_eq!(bin.join("mortar").canonicalize().unwrap(), binary);
    // Clear MORTAR_OP and all inherited Git overrides on every command, including commands made after install.
    Command::new("git").arg("-C").arg(root).args(args).env_clear().envs(env())
        .env("PATH", shim_path(bin)).env("HOME", root).env("TMPDIR", root)
        .stdin(Stdio::null()).output().unwrap()
}

fn git(root: &Path, args: &[&str]) -> String {
    let result = output(root, args);
    assert!(result.status.success(), "git {args:?}: {result:?}");
    String::from_utf8(result.stdout).unwrap().trim_end_matches('\n').to_string()
}

#[test]
fn installed_guard_allows_real_worktree_and_head_symrefs_but_refuses_an_unintended_integration_move() {
    let repo = TestRepo::new().unwrap();
    let root = repo.path.canonicalize().unwrap();
    let branch = git(&root, &["symbolic-ref", "HEAD"]);
    let origin_branch = branch.strip_prefix("refs/heads/").unwrap().to_string();
    git(&root, &["commit", "--allow-empty", "-m", "first"]);
    let old = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["branch", "integration"]);
    git(&root, &["commit", "--allow-empty", "-m", "second"]);
    let new = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(old, new);
    let refs = LandRefs { origin_branch, integration_branch: "integration".into(), remote: "origin".into() };
    let gitio = Git { program: "git".into(), repo: root.clone(), env: env() };
    land::guard_install(&gitio, &refs).unwrap();

    let linked = root.join("x");
    git(&root, &["worktree", "add", "-b", "x", linked.to_str().unwrap()]);
    assert!(linked.is_dir());
    assert_eq!(git(&root, &["rev-parse", "refs/heads/x"]), new);
    git(&root, &["symbolic-ref", "HEAD", "refs/heads/x"]);
    assert_eq!(git(&root, &["symbolic-ref", "HEAD"]), "refs/heads/x");

    let refused = output(&root, &["update-ref", "refs/heads/integration", &new]);
    assert!(!refused.status.success(), "unguarded direct integration update: {refused:?}");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("only Mortar moves refs/heads/integration"),
        "wrong refusal: {refused:?}");
    assert_eq!(git(&root, &["rev-parse", "refs/heads/integration"]), old);
    assert_eq!(git(&root, &["rev-parse", &branch]), new);
}

#[test]
fn installed_guard_allows_no_move_checkouts_of_both_guarded_branches_but_refuses_real_commits() {
    let repo = TestRepo::new().unwrap(); let root = repo.path.canonicalize().unwrap();
    assert_eq!(git(&root, &["symbolic-ref", "HEAD"]), "refs/heads/main");
    git(&root, &["commit", "--allow-empty", "-m", "first"]);
    let old = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["branch", "integration"]);
    git(&root, &["commit", "--allow-empty", "-m", "second"]);
    let new = git(&root, &["rev-parse", "HEAD"]); assert_ne!(old, new);
    let refs = LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() };
    let gitio = Git { program: "git".into(), repo: root.clone(), env: env() };
    land::guard_install(&gitio, &refs).unwrap();

    let integration = root.join("wi");
    git(&root, &["worktree", "add", "--force", integration.to_str().unwrap(), "integration"]);
    assert!(integration.is_dir()); assert_eq!(git(&integration, &["rev-parse", "HEAD"]), old);
    assert_eq!(git(&root, &["rev-parse", "refs/heads/integration"]), old);
    let main = root.join("wm");
    git(&root, &["worktree", "add", "--force", main.to_str().unwrap(), "main"]);
    assert!(main.is_dir()); assert_eq!(git(&main, &["rev-parse", "HEAD"]), new);
    assert_eq!(git(&root, &["rev-parse", "refs/heads/main"]), new);

    for (folder, reference, tip) in [(&integration, "refs/heads/integration", &old), (&main, "refs/heads/main", &new)] {
        let refused = output(folder, &["commit", "--allow-empty", "-m", "third"]);
        assert!(!refused.status.success(), "unintended commit on {reference}: {refused:?}");
        assert!(String::from_utf8_lossy(&refused.stderr).contains(&format!("only Mortar moves {reference}")),
            "wrong refusal on {reference}: {refused:?}");
        assert_eq!(git(&root, &["rev-parse", reference]), *tip);
        assert_eq!(git(folder, &["rev-parse", "HEAD"]), *tip);
    }
}
