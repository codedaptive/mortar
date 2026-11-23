use crate::{json, support::{fixture_partners::{partners, text, Partners}, raw, write}};
use gitio::Git;

#[test]
fn partner_status_reports_record_and_movement() {
    let f = Partners::new(); let base = f.set();
    let out = f.app("status"); let body = text(&out);
    let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), base.as_str());
    assert!(!status.flag("partner_moved"));
    assert!(body.contains(&partners(&f.web)), "{body}");
    let position = |key: &str| body.find(&format!("\"{key}\":")).unwrap();
    assert!(position("gaps") < position("base"));
    assert!(position("base") < position("partners"));
    assert!(position("partners") < position("partner_moved"));
    assert!(body.contains("\"savings\":"), "{body}");
    assert!(position("partner_moved") < position("savings"));
    f.advance("refs/heads/zone/web", &[("web/w.txt", b"new web\n")]);
    assert!(json::read(f.app("status"), "status", 0).flag("partner_moved"));
}

#[test]
fn partner_status_plain_has_empty_partners() {
    let f = Partners::new();
    json::check(f.cut(&[]), "folder", 0, "made");
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    let out = f.app("status"); let body = text(&out);
    assert!(body.contains("\"base\":"), "{body}");
    let status = json::read(out, "status", 0);
    assert_eq!(status.text("base"), record.base.as_str());
    assert!(body.contains("\"partners\":[]"));
    assert!(!status.flag("partner_moved"));
}

#[test]
fn partner_status_overlaps_fork_at_recorded_base() {
    let f = Partners::new(); f.set();
    f.commit_request("src/core/c.txt", b"request work\n");
    let peer = f.repo.path.join("peer");
    let out = f.command(&f.repo.path, &["--actor", "harness", "--json", "folder", "request", "core", "peer", peer.to_str().unwrap(), "--paths", "src/core"]);
    json::check(out, "folder", 0, "made");
    write(&peer, "web/w.txt", b"peer web\n");
    let git = Git { repo: peer, ..f.git.clone() };
    raw(&git, &["add", "--sparse", "web/w.txt"]);
    raw(&git, &["commit", "-qm", "peer"]);
    let status = json::read(f.app("status"), "status", 0);
    assert!(status.strings("overlaps").is_empty());
}

#[test]
fn partner_overlap_verb_forks_at_recorded_base() {
    let f = Partners::new(); f.set();
    f.commit_request("src/core/c.txt", b"request work\n");
    let peer = f.repo.path.join("peer");
    let out = f.command(&f.repo.path, &["--actor", "harness", "--json", "folder", "request", "core", "peer", peer.to_str().unwrap(), "--paths", "src/core"]);
    json::check(out, "folder", 0, "made");
    write(&peer, "web/w.txt", b"peer web\n");
    let git = Git { repo: peer, ..f.git.clone() };
    raw(&git, &["add", "--sparse", "web/w.txt"]);
    raw(&git, &["commit", "-qm", "peer"]);
    let report = json::read(f.command(&f.repo.path, &["--actor", "harness", "--json", "overlap", "core"]), "overlap", 0);
    assert!(report.strings("pairs").is_empty(), "{:?}", report.strings("pairs"));
    assert_eq!(report.text("outcome"), "no-overlap");
}
