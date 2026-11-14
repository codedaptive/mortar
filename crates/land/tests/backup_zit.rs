//! backup and restore_backup on throwaway repositories (SPEC-land 08): a work repository, a bare
//! remote made inside it at `remote.git`, and a fresh store on the same remote. Every repository is a testkit
//! TestRepo (or lives inside one) and is removed when it drops. Commits are written with fixed identities and
//! dates, so the same message gives the same commit id in any repository: the test sets up every expected id.
//! Each store's op log is recovered before its first backup or restore, as the caller's contract asks (S-16).
//! The plan_ cases prove the decisions of plan_backup and restore_plan that run another land file (S-102): leases,
//! refs outside this host's mapped backups, another host's backup, and the order of Backup.pushed and of restore's
//! entries, each against refs set by hand on the bare remote.

use gitio::{Git, Oid, Person, RefUpdate, RunOpts, UpdateRefsOutcome};
use land::{backup, restore_backup, Backup, LandRefs, Restored};
use oplog::{host_name, read_op_log, recover, OpPhase};
use testkit::TestRepo;

const ACTOR: &str = "zit@backup";

/// One repository and the Git that runs in it, with no system or global git config.
struct Store {
    repo: TestRepo,
    git: Git,
}

fn store() -> Store {
    let repo = TestRepo::new().unwrap();
    let mut git = Git::new(&repo.path);
    git.env = vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
    ];
    Store { repo, git }
}

/// A work store and the path of the bare remote made inside it, as LandRefs.
fn work_and_remote() -> (Store, LandRefs) {
    let work = store();
    let remote = work.repo.path.join("remote.git").to_str().unwrap().to_string();
    work.git.run(&["init", "-q", "--bare", remote.as_str()], &RunOpts::default()).unwrap();
    let refs = LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote };
    (work, refs)
}

/// A root commit of the empty tree with `message`; the same message gives the same id in every repository.
fn commit(git: &Git, message: &str) -> Oid {
    let opts = RunOpts { stdin: Some(Vec::new()), ..RunOpts::default() };
    let out = git.run(&["hash-object", "-t", "tree", "-w", "--stdin"], &opts).unwrap();
    let tree = Oid::parse(String::from_utf8(out.stdout).unwrap().trim_end()).unwrap();
    let who = Person { name: "Zit".into(), email: "zit@example.com".into(), date: Some("1700000000 +0000".into()) };
    git.commit_tree(&tree, &[], message, Some(&who), Some(&who)).unwrap()
}

fn set(git: &Git, refname: &str, new: &Oid) {
    let update = RefUpdate { refname: refname.into(), new: new.clone(), old: None };
    assert_eq!(git.update_refs(&[update]).unwrap(), UpdateRefsOutcome::Updated);
}

fn delete(git: &Git, refname: &str, old: &Oid) {
    let zero = Oid::parse("0000000000000000000000000000000000000000").unwrap();
    let update = RefUpdate { refname: refname.into(), new: zero, old: Some(old.clone()) };
    assert_eq!(git.update_refs(&[update]).unwrap(), UpdateRefsOutcome::Updated);
}

fn settle(git: &Git) {
    recover(git, &git.common_dir().unwrap()).unwrap();
}

fn host() -> String {
    host_name().unwrap()
}

/// The remote's refs under this host's backup namespace, read with ls-remote, sorted by name.
fn backed_up(git: &Git, refs: &LandRefs) -> Vec<(String, Oid)> {
    let prefix = format!("refs/mortar-backup/{}/", host());
    let mut listed: Vec<(String, Oid)> = git
        .ls_remote(&refs.remote, &[])
        .unwrap()
        .into_iter()
        .filter(|(_, name)| name.starts_with(&prefix))
        .map(|(oid, name)| (name, oid))
        .collect();
    listed.sort();
    listed
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
}

fn names(list: &[&str]) -> Vec<String> {
    sorted(list.iter().map(|n| n.to_string()).collect())
}

/// Restored entries in a stable order, so a comparison does not depend on the remote's listing order.
fn by_debug(mut entries: Vec<Restored>) -> Vec<Restored> {
    entries.sort_by_key(|e| format!("{e:?}"));
    entries
}

const ZONE: &str = "refs/heads/zone/a";
const REQ: &str = "refs/heads/req/land/r1";
const SNAP: &str = "refs/mortar/snap/s1";
const SQUASH: &str = "refs/mortar/squash/land";

/// The work store with one ref under each of the four backed-up prefixes, each at its own commit.
fn four_refs(git: &Git) -> [(&'static str, Oid); 4] {
    let refs = [(ZONE, commit(git, "zone a\n")), (REQ, commit(git, "req r1\n")), (SNAP, commit(git, "snap s1\n")),
        (SQUASH, commit(git, "squash land\n"))];
    for (name, oid) in &refs {
        set(git, name, oid);
    }
    refs
}

#[test]
fn a_backup_restores_every_ref_into_a_fresh_store() {
    let (work, refs) = work_and_remote();
    let made = four_refs(&work.git);
    settle(&work.git);
    let done = backup(&work.git, &refs, ACTOR).unwrap();
    assert_eq!(sorted(done.pushed), names(&[ZONE, REQ, SNAP, SQUASH]));
    assert!(done.unchanged.is_empty());

    let fresh = store();
    settle(&fresh.git);
    let restored = restore_backup(&fresh.git, &refs, &host(), ACTOR).unwrap();
    let expected = made.iter().map(|(name, oid)| Restored::Created { refname: name.to_string(), oid: oid.clone() });
    assert_eq!(by_debug(restored), by_debug(expected.collect()));
    for (name, oid) in &made {
        assert_eq!(fresh.git.rev_parse(name).unwrap(), Some(oid.clone()), "{name}");
    }
}

#[test]
fn a_second_backup_pushes_nothing() {
    let (work, refs) = work_and_remote();
    four_refs(&work.git);
    settle(&work.git);
    backup(&work.git, &refs, ACTOR).unwrap();
    let again = backup(&work.git, &refs, ACTOR).unwrap();
    assert!(again.pushed.is_empty(), "{:?}", again.pushed);
    assert_eq!(sorted(again.unchanged), names(&[ZONE, REQ, SNAP, SQUASH]));
}

#[test]
fn a_ref_deleted_locally_is_deleted_from_the_backup() {
    let (work, refs) = work_and_remote();
    let made = four_refs(&work.git);
    settle(&work.git);
    backup(&work.git, &refs, ACTOR).unwrap();
    delete(&work.git, ZONE, &made[0].1);
    let next = backup(&work.git, &refs, ACTOR).unwrap();
    assert_eq!(next.pushed, names(&[ZONE]));
    assert_eq!(sorted(next.unchanged), names(&[REQ, SNAP, SQUASH]));
    let prefix = format!("refs/mortar-backup/{}/", host());
    let expected = vec![
        (format!("{prefix}heads/req/land/r1"), made[1].1.clone()),
        (format!("{prefix}mortar/snap/s1"), made[2].1.clone()),
        (format!("{prefix}mortar/squash/land"), made[3].1.clone()),
    ];
    assert_eq!(backed_up(&work.git, &refs), expected);
}

#[test]
fn restore_lists_a_differing_ref_without_moving_it_and_an_equal_one_as_same() {
    let (work, refs) = work_and_remote();
    let made = four_refs(&work.git);
    settle(&work.git);
    backup(&work.git, &refs, ACTOR).unwrap();

    let fresh = store();
    let other = commit(&fresh.git, "fresh zone a\n");
    set(&fresh.git, ZONE, &other);
    set(&fresh.git, SNAP, &commit(&fresh.git, "snap s1\n"));
    settle(&fresh.git);
    let restored = restore_backup(&fresh.git, &refs, &host(), ACTOR).unwrap();
    let expected = vec![
        Restored::Differs { refname: ZONE.into(), local: other.clone(), backup: made[0].1.clone() },
        Restored::Created { refname: REQ.into(), oid: made[1].1.clone() },
        Restored::Same { refname: SNAP.into() },
        Restored::Created { refname: SQUASH.into(), oid: made[3].1.clone() },
    ];
    assert_eq!(by_debug(restored), by_debug(expected));
    assert_eq!(fresh.git.rev_parse(ZONE).unwrap(), Some(other));
    assert_eq!(fresh.git.rev_parse(SNAP).unwrap(), Some(made[2].1.clone()));
}

#[test]
fn refs_outside_the_four_prefixes_are_never_backed_up() {
    let (work, refs) = work_and_remote();
    let zone = commit(&work.git, "zone a\n");
    set(&work.git, ZONE, &zone);
    set(&work.git, "refs/heads/main", &commit(&work.git, "main\n"));
    set(&work.git, "refs/heads/other", &commit(&work.git, "other\n"));
    set(&work.git, "refs/heads/zones/x", &commit(&work.git, "zones x\n"));
    settle(&work.git);
    let done = backup(&work.git, &refs, ACTOR).unwrap();
    assert_eq!(sorted(done.pushed), names(&[ZONE]));
    let prefix = format!("refs/mortar-backup/{}/", host());
    assert_eq!(backed_up(&work.git, &refs), vec![(format!("{prefix}heads/zone/a"), zone)]);
    assert_eq!(work.git.ls_remote(&refs.remote, &[]).unwrap().len(), 1);
}

#[test]
fn a_pushing_backup_logs_an_intent_then_done_and_an_idle_one_logs_nothing() {
    let (work, refs) = work_and_remote();
    four_refs(&work.git);
    settle(&work.git);
    let common = work.git.common_dir().unwrap();
    backup(&work.git, &refs, ACTOR).unwrap();
    let log = read_op_log(&common).unwrap().records;
    let ours: Vec<_> = log.iter().filter(|r| r.kind == "land-backup").collect();
    assert_eq!(ours.iter().map(|r| r.phase).collect::<Vec<_>>(), vec![OpPhase::Intent, OpPhase::Done]);
    assert_eq!(ours[0].op_id, ours[1].op_id);
    assert_eq!(ours[0].actor, ACTOR);

    backup(&work.git, &refs, ACTOR).unwrap();
    assert_eq!(read_op_log(&common).unwrap().records.len(), log.len());
}

/// Git in the bare remote at `refs.remote`, with no system or global git config.
fn remote_git(refs: &LandRefs) -> Git {
    let mut git = Git::new(&refs.remote);
    git.env = vec![
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
    ];
    git
}

/// `name` in the remote at a commit of `message`, made there; the commit's id.
fn set_on_remote(remote: &Git, name: &str, message: &str) -> Oid {
    let oid = commit(remote, message);
    set(remote, name, &oid);
    oid
}

const OTHER_HOST: &str = "zit-other-host";

#[test]
fn plan_a_moved_ref_is_pushed_again_over_its_old_backup() {
    let (work, refs) = work_and_remote();
    let first = commit(&work.git, "zone a\n");
    set(&work.git, ZONE, &first);
    settle(&work.git);
    backup(&work.git, &refs, ACTOR).unwrap();
    let moved = commit(&work.git, "zone a moved\n");
    delete(&work.git, ZONE, &first);
    set(&work.git, ZONE, &moved);
    // A set under lease Absent would be refused here, the backup ref being present: Ok proves the lease at the old id.
    let again = backup(&work.git, &refs, ACTOR).unwrap();
    assert_eq!(again, Backup { pushed: vec![ZONE.to_string()], unchanged: vec![] });
    let prefix = format!("refs/mortar-backup/{}/", host());
    assert_eq!(backed_up(&work.git, &refs), vec![(format!("{prefix}heads/zone/a"), moved)]);
}

#[test]
fn plan_remote_refs_outside_this_hosts_mapped_backups_are_never_pushed_or_deleted() {
    let (work, refs) = work_and_remote();
    let zone = commit(&work.git, "zone a\n");
    set(&work.git, ZONE, &zone);
    let remote = remote_git(&refs);
    let prefix = format!("refs/mortar-backup/{}/", host());
    let longer_host = format!("refs/mortar-backup/{}0/", host());
    let strays: Vec<(String, Oid)> = [
        ("refs/heads/main".to_string(), "remote main\n"),
        (format!("{prefix}heads/main"), "backed-up main\n"),
        (format!("{prefix}tags/v1"), "backed-up tag\n"),
        (format!("{longer_host}heads/zone/a"), "longer host zone a\n"),
        (format!("{longer_host}heads/zone/gone"), "longer host zone gone\n"),
    ]
    .into_iter()
    .map(|(name, message)| {
        let oid = set_on_remote(&remote, &name, message);
        (name, oid)
    })
    .collect();
    settle(&work.git);
    let done = backup(&work.git, &refs, ACTOR).unwrap();
    assert_eq!(done, Backup { pushed: vec![ZONE.to_string()], unchanged: vec![] });
    assert_eq!(remote.read_ref(&format!("{prefix}heads/zone/a")).unwrap(), Some(zone));
    for (name, oid) in &strays {
        assert_eq!(remote.read_ref(name).unwrap(), Some(oid.clone()), "{name}");
    }
}

#[test]
fn plan_another_hosts_backup_of_the_same_ref_is_left_and_this_hosts_is_pushed() {
    let (work, refs) = work_and_remote();
    let zone = commit(&work.git, "zone a\n");
    set(&work.git, ZONE, &zone);
    let remote = remote_git(&refs);
    let other = format!("refs/mortar-backup/{OTHER_HOST}/heads/zone/a");
    let others = set_on_remote(&remote, &other, "other host zone a\n");
    settle(&work.git);
    // Counted as this host's backup, its id would lease the set and the push of an absent ref would be refused.
    let done = backup(&work.git, &refs, ACTOR).unwrap();
    assert_eq!(done, Backup { pushed: vec![ZONE.to_string()], unchanged: vec![] });
    assert_eq!(remote.read_ref(&format!("refs/mortar-backup/{}/heads/zone/a", host())).unwrap(), Some(zone.clone()));
    assert_eq!(remote.read_ref(&other).unwrap(), Some(others));
}

#[test]
fn plan_pushed_lists_sets_in_local_order_then_deletes_in_remote_order() {
    let (work, refs) = work_and_remote();
    let zone_b = ("refs/heads/zone/b", commit(&work.git, "zone b\n"));
    let req_2 = ("refs/heads/req/land/r2", commit(&work.git, "req r2\n"));
    for (name, oid) in [&zone_b, &req_2] {
        set(&work.git, name, oid);
    }
    settle(&work.git);
    backup(&work.git, &refs, ACTOR).unwrap();
    delete(&work.git, zone_b.0, &zone_b.1);
    delete(&work.git, req_2.0, &req_2.1);
    four_refs(&work.git);
    let done = backup(&work.git, &refs, ACTOR).unwrap();
    // Local order is BACKUP_PREFIXES' (zone, req, snap, squash); the remote lists heads/req before heads/zone.
    assert_eq!(done.pushed, vec![ZONE, REQ, SNAP, SQUASH, req_2.0, zone_b.0]);
    assert!(done.unchanged.is_empty(), "{:?}", done.unchanged);
}

#[test]
fn plan_restore_of_a_backed_up_main_gives_nothing_and_creates_nothing() {
    let (_work, refs) = work_and_remote();
    set_on_remote(&remote_git(&refs), &format!("refs/mortar-backup/{}/heads/main", host()), "main\n");
    let fresh = store();
    settle(&fresh.git);
    assert_eq!(restore_backup(&fresh.git, &refs, &host(), ACTOR).unwrap(), vec![]);
    assert_eq!(fresh.git.read_ref("refs/heads/main").unwrap(), None);
}

#[test]
fn plan_restore_skips_refs_outside_this_hosts_backup() {
    let (_work, refs) = work_and_remote();
    let remote = remote_git(&refs);
    set_on_remote(&remote, &format!("refs/mortar-backup/{OTHER_HOST}/heads/zone/b"), "other zone b\n");
    // With refs/heads/main on the remote, ls-remote also lists HEAD.
    set_on_remote(&remote, "refs/heads/main", "remote main\n");
    set_on_remote(&remote, "refs/heads/zone/a", "remote zone a\n");
    set_on_remote(&remote, &format!("refs/mortar-backup/{}/tags/v1", host()), "backed-up tag\n");
    set_on_remote(&remote, &format!("refs/mortar-backup/{}0/heads/zone/c", host()), "longer host zone c\n");
    let fresh = store();
    settle(&fresh.git);
    assert_eq!(restore_backup(&fresh.git, &refs, &host(), ACTOR).unwrap(), vec![]);
    for name in ["refs/heads/main", "refs/heads/zone/a", "refs/heads/zone/b", "refs/heads/zone/c", "refs/tags/v1"] {
        assert_eq!(fresh.git.read_ref(name).unwrap(), None, "{name}");
    }
}

#[test]
fn plan_restore_entries_come_in_remote_order() {
    let (work, refs) = work_and_remote();
    let made = four_refs(&work.git);
    settle(&work.git);
    backup(&work.git, &refs, ACTOR).unwrap();
    let fresh = store();
    let other = commit(&fresh.git, "fresh zone a\n");
    set(&fresh.git, ZONE, &other);
    set(&fresh.git, SNAP, &commit(&fresh.git, "snap s1\n"));
    settle(&fresh.git);
    // The remote lists heads/req before heads/zone; local order would put zone first.
    let expected = vec![
        Restored::Created { refname: REQ.into(), oid: made[1].1.clone() },
        Restored::Differs { refname: ZONE.into(), local: other, backup: made[0].1.clone() },
        Restored::Same { refname: SNAP.into() },
        Restored::Created { refname: SQUASH.into(), oid: made[3].1.clone() },
    ];
    assert_eq!(restore_backup(&fresh.git, &refs, &host(), ACTOR).unwrap(), expected);
}
