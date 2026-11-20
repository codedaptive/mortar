//! search_first_failing over pass/fail tables: the break found for every size and break index, each commit
//! probed once within 2 + ceil(log2(n)) probes, the two NotFound ends, a probe error ending the search, and a
//! non-monotonic table still answering a fail-after-pass boundary.

use std::collections::{HashMap, HashSet};

use gitio::Oid;

use super::{search_first_failing, BisectNotFound, BisectVerdict};

fn oid(i: usize) -> Oid {
    Oid::parse(&format!("{i:040x}")).unwrap()
}

/// last_pass is oid(0); candidates are oid(1..=n).
fn candidates(n: usize) -> Vec<Oid> {
    (1..=n).map(oid).collect()
}

/// Runs the search over `table`, returning the verdict and every commit probed, in order.
fn run(last_pass: &Oid, cands: &[Oid], table: &HashMap<Oid, bool>) -> (BisectVerdict, Vec<Oid>) {
    let mut probed = Vec::new();
    let verdict = search_first_failing(last_pass, cands, |c: &Oid| {
        probed.push(c.clone());
        Ok::<bool, ()>(table[c])
    })
    .unwrap();
    (verdict, probed)
}

fn smallest_power_at_least(n: usize) -> usize {
    (0..).find(|p| 1usize << p >= n).unwrap()
}

#[test]
fn every_break_in_every_size_is_found_within_the_probe_bound() {
    for n in 1..=17 {
        let cands = candidates(n);
        for k in 0..n {
            let mut table: HashMap<Oid, bool> = cands.iter().enumerate().map(|(i, c)| (c.clone(), i < k)).collect();
            table.insert(oid(0), true);
            let (verdict, probed) = run(&oid(0), &cands, &table);
            assert_eq!(verdict, BisectVerdict::FirstFailing(cands[k].clone()), "n={n} k={k}");
            let distinct: HashSet<&Oid> = probed.iter().collect();
            assert_eq!(distinct.len(), probed.len(), "n={n} k={k}: a commit probed twice: {probed:?}");
            let bound = 2 + smallest_power_at_least(n);
            assert!(probed.len() <= bound, "n={n} k={k}: {} probes, bound {bound}", probed.len());
        }
    }
}

#[test]
fn failed_passing_is_failed_passes_after_one_probe() {
    let cands = candidates(5);
    let table: HashMap<Oid, bool> = (0..=5).map(|i| (oid(i), true)).collect();
    let (verdict, probed) = run(&oid(0), &cands, &table);
    assert_eq!(verdict, BisectVerdict::NotFound(BisectNotFound::FailedPasses));
    assert_eq!(probed, vec![oid(5)]);
}

#[test]
fn last_pass_failing_is_last_pass_fails_after_failed_then_last_pass() {
    let cands = candidates(5);
    let table: HashMap<Oid, bool> = (0..=5).map(|i| (oid(i), false)).collect();
    let (verdict, probed) = run(&oid(0), &cands, &table);
    assert_eq!(verdict, BisectVerdict::NotFound(BisectNotFound::LastPassFails));
    assert_eq!(probed, vec![oid(5), oid(0)]);
}

#[test]
fn no_candidates_and_last_pass_passing_is_failed_passes() {
    let table = HashMap::from([(oid(0), true)]);
    let (verdict, probed) = run(&oid(0), &[], &table);
    assert_eq!(verdict, BisectVerdict::NotFound(BisectNotFound::FailedPasses));
    assert_eq!(probed, vec![oid(0)]);
}

#[test]
fn no_candidates_and_last_pass_failing_is_last_pass_fails() {
    let table = HashMap::from([(oid(0), false)]);
    let (verdict, probed) = run(&oid(0), &[], &table);
    assert_eq!(verdict, BisectVerdict::NotFound(BisectNotFound::LastPassFails));
    assert_eq!(probed, vec![oid(0)]);
}

#[test]
fn a_probe_error_is_returned_and_no_probe_follows_it() {
    // 8 candidates breaking at index 3 take 2 + 3 = 5 probes; fail each one in turn.
    let cands = candidates(8);
    for failing_probe in 1..=5 {
        let mut count = 0;
        let result = search_first_failing(&oid(0), &cands, |c: &Oid| {
            count += 1;
            if count == failing_probe {
                return Err(format!("probe {count} broke"));
            }
            Ok(*c == oid(0) || *c < oid(4))
        });
        assert_eq!(result, Err(format!("probe {failing_probe} broke")));
        assert_eq!(count, failing_probe, "a probe followed the error");
    }
}

#[test]
fn a_non_monotonic_table_answers_a_failing_commit_whose_predecessor_passed() {
    // last_pass P, then P F P F F: the fail-after-pass boundaries are oid(2) and oid(4).
    let cands = candidates(5);
    let table = HashMap::from([
        (oid(0), true),
        (oid(1), true),
        (oid(2), false),
        (oid(3), true),
        (oid(4), false),
        (oid(5), false),
    ]);
    let (verdict, _) = run(&oid(0), &cands, &table);
    assert!(
        [BisectVerdict::FirstFailing(oid(2)), BisectVerdict::FirstFailing(oid(4))].contains(&verdict),
        "{verdict:?} is not a fail-after-pass boundary"
    );
}
