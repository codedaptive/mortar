//! The cases of 03-kill.md and each one's checks, in order. A const only: case_lines_tests.rs and
//! kill_not_run_tests.rs read it through CaseLines.

/// Each case's label, then its checks' titles; 03's "Consistent" list (CONSISTENT_ITEMS) follows each case. Case 3
/// is two: killed before, then killed after, its integration move.
pub const KILL_CASES: [(&str, &[&str]); 5] = [
    (
        "1.",
        &[
            "mr join in r1 (a fast-forward), killed before its ref transaction: the op log's open intent is that join",
            "zone/a unmoved",
            "mr join in r1 again finishes",
            "mr log: the killed op closed failed (not-moved) by recover, and one done join",
            "the zone moved once",
        ],
    ),
    (
        "2.",
        &[
            "the zone branch moved",
            "mr join in r2 (a rebase), killed after its ref transaction: the op log's open intent is that join",
            "the request ref moved",
            "mr status in r2 reports the stalled move, next: mr join",
            "mr join in r2 finishes the folder move",
            "mr log: the killed op closed done by recover, and no second move of the request ref",
        ],
    ),
    (
        "3. before:",
        &[
            "mr land <squash of zone a>, killed before its integration move: the op log's open intent is that land",
            "mr land <same commit> again: landed",
            "integration holds the squash once (one landing commit)",
            "mr log: the killed op closed failed (not-moved) by recover, and one done land",
        ],
    ),
    (
        "3. after:",
        &[
            "mr land <squash of zone b>, killed after its integration move: the op log's open intent is that land",
            "mr land <same commit> again: landed",
            "integration holds the squash once (one landing commit)",
            "mr log: the killed op closed done by recover, and one done land",
        ],
    ),
    (
        "4.",
        &[
            "mr publish, killed between its push and its local move: the op log's open intent is that publish's",
            "the landing state says a publish was interrupted",
            "mr publish again: published",
            "origin's main and the local main at the landed commit, once",
            "the push intent closed done by publish (S-103), not by recover",
        ],
    ),
];
