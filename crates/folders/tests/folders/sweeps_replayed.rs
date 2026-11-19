//! sweep_refs and disk's sweep figure take the replay-record refs
//! (refs/mortar/replayed/<zone>/<source>) of zones not in the caller's zone list, and keep a listed zone's; real git
//! and the real op log in a throwaway repository.
use super::maintenance_support::*;
use crate::support::run;
use folders::disk;
use gitio::Oid;

#[test]
fn sweep_deletes_unlisted_zones_replayed_refs_in_one_op_keeps_listed_and_disk_counted_exactly_those() {
    let m = Removal::new();
    let (s, t, u) = (child(&m.f, "source s"), child(&m.f, "source t"), child(&m.f, "source u"));
    let zones = vec!["red".to_string(), "team/land".to_string()];
    run(&m.f.git, &["update-ref", "refs/heads/integration", m.f.head.as_str()]);
    let kept = [format!("refs/mortar/replayed/red/{}", s.as_str()), format!("refs/mortar/replayed/team/land/{}", u.as_str())];
    // list_refs sorts by refname: blue/<s> and blue/<t> by their ids, then team/<u>.
    let mut blue = [format!("refs/mortar/replayed/blue/{}", s.as_str()), format!("refs/mortar/replayed/blue/{}", t.as_str())];
    blue.sort();
    let stale = [blue[0].clone(), blue[1].clone(), format!("refs/mortar/replayed/team/{}", u.as_str())];
    for name in stale.iter().chain(&kept) { run(&m.f.git, &["update-ref", name, m.f.head.as_str()]); }
    let main = String::from_utf8(run(&m.f.git, &["symbolic-ref", "HEAD"])).unwrap().trim_end().to_string();
    let mut expected = refs(&m.f.git);
    for name in &stale { expected.remove(name); }
    let report = disk(&m.f.git, 1, &zones, &[], &[]).unwrap();
    let start = records(&m.f).len();
    let deleted = sweep(&m.f, &m.f.git, &zones, &[], &[]).unwrap();
    assert_eq!(deleted, stale.to_vec());
    assert_eq!(refs(&m.f.git), expected);
    for name in &kept { assert_eq!(m.f.git.read_ref(name).unwrap(), Some(m.f.head.clone()), "{name} is a listed zone's"); }
    assert_eq!(m.f.git.read_ref(&main).unwrap(), Some(m.f.head.clone()));
    assert_eq!(m.f.git.read_ref("refs/heads/integration").unwrap(), Some(m.f.head.clone()));
    let names = stale.iter().map(String::as_str).collect::<Vec<_>>();
    assert_delete_pair(&records(&m.f)[start..], "folder-sweep-refs", &names, &m.f.head);
    assert_eq!(swept_ref_count(&report.sweep.reason), 3);
    assert_eq!(swept_ref_count(&report.sweep.reason), deleted.len());
}

#[test]
fn a_zone_dropped_from_the_list_loses_its_replayed_refs_in_the_next_sweeps_own_op() {
    let m = Removal::new();
    let s = child(&m.f, "source s");
    let red = format!("refs/mortar/replayed/red/{}", s.as_str());
    let land = format!("refs/mortar/replayed/team/land/{}", s.as_str());
    for name in [&red, &land] { run(&m.f.git, &["update-ref", name, m.f.head.as_str()]); }
    let both = vec!["red".to_string(), "team/land".to_string()];
    assert_eq!(sweep(&m.f, &m.f.git, &both, &[], &[]).unwrap(), Vec::<String>::new());
    let start = records(&m.f).len();
    assert_eq!(sweep(&m.f, &m.f.git, &["team/land".to_string()], &[], &[]).unwrap(), vec![red.clone()]);
    assert_eq!(m.f.git.read_ref(&red).unwrap(), None::<Oid>);
    assert_eq!(m.f.git.read_ref(&land).unwrap(), Some(m.f.head.clone()), "team/land is still listed");
    assert_delete_pair(&records(&m.f)[start..], "folder-sweep-refs", &[red.as_str()], &m.f.head);
}

/// The ref count sweep_figure writes into its reason: "... only from the <n> refs a ref sweep deletes".
fn swept_ref_count(reason: &Option<String>) -> usize {
    let (_, tail) = reason.as_deref().unwrap().split_once("only from the ").unwrap();
    tail.split_once(" refs a ref sweep deletes").unwrap().0.parse().unwrap()
}
