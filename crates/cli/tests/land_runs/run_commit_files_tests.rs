//! commit-files in the integration folder, before and after a real landing and publish.
use std::path::PathBuf;
use crate::{json::check, support::{raw, write, Fixture, ZONE}};
use gitio::Git;

const INTEGRATION: &str = "refs/heads/integration";

struct Setup { f: Fixture, folder: PathBuf, bare: PathBuf }

impl Setup {
    fn new() -> Self {
        let f = Fixture::new();
        raw(&f.git, &["checkout", "-q", "--detach"]);
        raw(&f.git, &["branch", "integration", f.base.as_str()]);
        let folder = f.root.join("integration");
        raw(&f.git, &["worktree", "add", "-q", folder.to_str().unwrap(), "integration"]);
        let bare = f.root.join("origin.git");
        raw(&f.git, &["init", "-q", "--bare", "--initial-branch=main", bare.to_str().unwrap()]);
        raw(&f.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
        raw(&f.git, &["push", "-q", "origin", "refs/heads/main:refs/heads/main"]);
        Self { f, folder, bare }
    }
}

#[test]
fn commit_files_after_land_is_parented_by_the_landing_and_publish_pushes_it() {
    let s = Setup::new();
    let zone = s.f.commit(&s.f.base, &[("src/file", b"zone work\n")], "zone work\n");
    raw(&s.f.git, &["update-ref", ZONE, zone.as_str()]);
    let squash = check(s.f.command(&s.f.git.repo, &["--json", "--actor", "integrator", "squash", "core"])
        .output().unwrap(), "squash", 0, "squashed");
    check(s.f.command(&s.f.git.repo, &["--json", "--actor", "integrator", "land", squash.text("commit")])
        .output().unwrap(), "land", 0, "landed");
    let landed = s.f.git.read_ref(INTEGRATION).unwrap().unwrap();
    write(&s.folder, "src/receipt", b"integration receipt\n");
    let r = check(s.f.command(&s.folder, &["--json", "--actor", "integrator", "commit-files", "src/receipt", "--message", "record receipt"])
        .output().unwrap(), "commit-files", 0, "committed");
    let tip = s.f.git.read_ref(INTEGRATION).unwrap().unwrap();
    assert_ne!(tip, landed);
    assert_eq!(r.text("commit"), tip.as_str());
    assert_eq!(r.text("ref"), INTEGRATION);
    assert!(r.flag("moved"));
    assert_eq!(s.f.git.commit_info(&tip).unwrap().parents, vec![landed.clone()]);
    assert_eq!(raw(&s.f.git, &["diff", "--name-status", landed.as_str(), tip.as_str()]), b"A\tsrc/receipt\n");
    assert_eq!(s.f.blob(&tip, "src/receipt"), b"integration receipt\n");
    assert_eq!(raw(&s.f.git, &["show", "-s", "--format=%s", tip.as_str()]), b"record receipt\n");
    check(s.f.command(&s.f.git.repo, &["--json", "--actor", "integrator", "publish"])
        .output().unwrap(), "publish", 0, "published");
    let remote = Git { repo: s.bare.clone(), ..s.f.git.clone() };
    assert_eq!(remote.read_ref("refs/heads/main").unwrap(), Some(tip));
}

#[test]
fn commit_files_without_a_landing_is_refused_with_land_next() {
    let s = Setup::new();
    let old = s.f.git.read_ref(INTEGRATION).unwrap().unwrap();
    write(&s.folder, "src/receipt", b"integration receipt\n");
    let r = check(s.f.command(&s.folder, &["--json", "--actor", "integrator", "commit-files", "src/receipt"])
        .output().unwrap(), "commit-files", 1, "refused");
    assert!(r.text("next").contains("mr land"));
    assert_eq!(s.f.git.read_ref(INTEGRATION).unwrap(), Some(old));
}
