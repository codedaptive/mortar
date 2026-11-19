//! CLI run_disk -> real Folders disk/Gitio store and worktree measurements, with independent byte oracles.
use super::fixture::*;
use crate::support::write;

#[test]
fn disk_lists_real_zone_and_request_bytes_counts_the_shared_store_once_and_never_sweeps_builds() {
    let w = World::new(); write(&w.s.f.request, "src/extra", b"request");
    write(&w.s.f.request, "target/debug/out", b"1234567890123"); write(&w.s.f.zone, "target/out", b"1234567");
    // The unrecorded main checkout must not contribute another folder, or have its target counted/swept.
    write(&w.s.f.git.repo, "target/out", b"main output is not a Mortar folder");
    let store = bytes_in_tree(&w.s.f.common.join("objects")); assert!(store > 0);
    let before = observe(&w); let r = app(&w, &["disk"], 0, "measured");
    let mut paths = r.strings("folders").to_vec(); paths.sort();
    let mut expected = vec![w.s.f.zone.display().to_string(), w.s.f.request.display().to_string(), w.b.display().to_string()]; expected.sort();
    assert_eq!(paths, expected);
    for line in [
        format!("{}: the zone folder of zone core; files 15 bytes, build 7 bytes", w.s.f.zone.display()),
        format!("{}: the zone folder of zone b; files 15 bytes, build 0 bytes", w.b.display()),
        format!("{}: the request folder of request task in zone core; files 22 bytes, build 13 bytes", w.s.f.request.display()),
    ] { assert!(r.strings("lines").contains(&line), "missing {line}: {r:?}"); }
    assert_eq!(r.number("store_bytes"), store);
    assert_eq!(r.strings("lines").iter().filter(|s| s.starts_with("object store:")).cloned().collect::<Vec<_>>(), vec![format!("object store: {store} bytes, counted once")]);
    assert_eq!(r.strings("refs"), &["refs/heads 2", "refs/heads/req 1", "refs/heads/zone 2", "refs/remotes/origin 1"]);
    assert_eq!(r.number("sweep_bytes"), 0, "sweep policy: no stale refs and no implicit build sweep");
    assert!(r.strings("lines").contains(&"an mr sweep with no --idle-days frees up to 0 bytes".into()));
    assert_eq!(r.text("next"), "mr sweep --idle-days <n> --actor <name>"); assert_eq!(observe(&w), before);
    // The same object store, not a per-cwd or per-folder sum, when called from the request instead of main.
    let again = run(&w, &w.s.f.request, &["disk"], 0, "measured");
    assert_eq!(again.number("store_bytes"), store); assert_eq!(again.strings("lines"), r.strings("lines"));
    assert_eq!(observe(&w), before);
}
