//! Zone integration tests of drop_plan, keep_plan and attempts_remaining, with unsaved_attempts / orphan_attempts
//! used for real (S-79 as amended by #1264), and of orphan_attempts, attempts_of and closed_snap_refs, which use
//! plan/attempt_number.rs for real (#1264).

mod drop_plan {
    //! drop_plan: any orphan folder refused first, all named in k order, saved or not; then unsaved attempts
    //! (folder or not) refused in k order; every attempt folder to remove and every branch to delete, in k order; no
    //! attempts plan nothing.

    use std::path::PathBuf;

    use folders::attempts::{drop_plan, Attempt, Dropped};
    use folders::FolderError;

    use crate::ref_pairs::oid;

    const A: &str = "1111111111111111111111111111111111111111";
    const B: &str = "2222222222222222222222222222222222222222";
    const C: &str = "3333333333333333333333333333333333333333";

    fn attempt(k: u32, hex: &str, folder: Option<&str>) -> Attempt {
        Attempt {
            k,
            branch: format!("refs/heads/try/z/r/{k}"),
            tip: oid(hex),
            folder: folder.map(PathBuf::from),
        }
    }

    #[test]
    fn drops_three_saved_attempts_in_k_order_whatever_the_input_order() {
        let attempts = vec![attempt(3, C, Some("/t/try-3")), attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2"))];
        let dropped = drop_plan(&attempts, &[], &[(3, true), (1, true), (2, true)]).unwrap();
        assert_eq!(
            dropped,
            Dropped {
                remove: vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-2"), PathBuf::from("/t/try-3")],
                delete: vec![
                    ("refs/heads/try/z/r/1".to_string(), oid(A)),
                    ("refs/heads/try/z/r/2".to_string(), oid(B)),
                    ("refs/heads/try/z/r/3".to_string(), oid(C)),
                ],
            }
        );
    }

    #[test]
    fn attempts_without_folders_are_skipped_from_remove_but_deleted() {
        let attempts = vec![attempt(1, A, None), attempt(2, B, Some("/t/try-2"))];
        let dropped = drop_plan(&attempts, &[], &[(1, true), (2, true)]).unwrap();
        assert_eq!(
            dropped,
            Dropped {
                remove: vec![PathBuf::from("/t/try-2")],
                delete: vec![("refs/heads/try/z/r/1".to_string(), oid(A)), ("refs/heads/try/z/r/2".to_string(), oid(B))],
            }
        );
    }

    #[test]
    fn unlisted_and_false_attempts_are_refused_in_k_order() {
        let attempts = vec![attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2")), attempt(3, C, Some("/t/try-3"))];
        match drop_plan(&attempts, &[], &[(2, true), (3, false)]) {
            Err(FolderError::AttemptsUnsaved { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3")])
            }
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn an_unsaved_folderless_attempt_is_refused_naming_its_branch() {
        let attempts = vec![attempt(1, A, None), attempt(2, B, Some("/t/try-2"))];
        match drop_plan(&attempts, &[], &[(2, true)]) {
            Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![PathBuf::from("refs/heads/try/z/r/1")]),
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn no_attempts_plan_nothing() {
        assert_eq!(drop_plan(&[], &[], &[]).unwrap(), Dropped { remove: Vec::new(), delete: Vec::new() });
    }

    #[test]
    fn orphans_are_refused_before_unsaved_attempts_naming_every_orphan_folder() {
        let attempts = vec![attempt(1, A, Some("/t/try-1"))];
        let orphans = vec![(2, PathBuf::from("/t/try-2")), (3, PathBuf::from("/t/try-3"))];
        match drop_plan(&attempts, &orphans, &[(3, true), (2, false)]) {
            Err(FolderError::OrphanAttempt { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("/t/try-2"), PathBuf::from("/t/try-3")])
            }
            other => panic!("expected OrphanAttempt, got {other:?}"),
        }
    }

    #[test]
    fn a_saved_orphan_is_still_refused() {
        let attempts = vec![attempt(3, C, Some("/t/try-3"))];
        let orphans = vec![(1, PathBuf::from("/t/try-1"))];
        match drop_plan(&attempts, &orphans, &[(1, true), (3, true)]) {
            Err(FolderError::OrphanAttempt { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-1")]),
            other => panic!("expected OrphanAttempt, got {other:?}"),
        }
    }

    #[test]
    fn orphans_alone_are_refused_in_k_order() {
        let orphans = vec![(4, PathBuf::from("/t/try-4")), (2, PathBuf::from("/t/try-2"))];
        match drop_plan(&[], &orphans, &[(2, true), (4, true)]) {
            Err(FolderError::OrphanAttempt { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("/t/try-2"), PathBuf::from("/t/try-4")])
            }
            other => panic!("expected OrphanAttempt, got {other:?}"),
        }
    }
}

mod keep_plan {
    //! keep_plan: a missing attempt (an orphan k included) refused, then any orphan folder, then unsaved dropped
    //! attempts (folder or not); the kept tip, the folders to remove (dropped attempts', the kept one last) and every
    //! branch to delete, in k order.

    use std::path::PathBuf;

    use folders::attempts::{keep_plan, Attempt, Kept};
    use folders::FolderError;

    use crate::ref_pairs::oid;

    const A: &str = "1111111111111111111111111111111111111111";
    const B: &str = "2222222222222222222222222222222222222222";
    const C: &str = "3333333333333333333333333333333333333333";

    fn attempt(k: u32, hex: &str, folder: Option<&str>) -> Attempt {
        Attempt {
            k,
            branch: format!("refs/heads/try/z/r/{k}"),
            tip: oid(hex),
            folder: folder.map(PathBuf::from),
        }
    }

    fn three() -> Vec<Attempt> {
        vec![attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2")), attempt(3, C, Some("/t/try-3"))]
    }

    #[test]
    fn keeps_attempt_2_removing_it_last_and_deleting_every_branch() {
        let kept = keep_plan("try/z/r/2", 2, &three(), &[], &[(1, true), (3, true)]).unwrap();
        assert_eq!(
            kept,
            Kept {
                tip: oid(B),
                remove: vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3"), PathBuf::from("/t/try-2")],
                delete: vec![
                    ("refs/heads/try/z/r/1".to_string(), oid(A)),
                    ("refs/heads/try/z/r/2".to_string(), oid(B)),
                    ("refs/heads/try/z/r/3".to_string(), oid(C)),
                ],
            }
        );
    }

    #[test]
    fn plans_in_k_order_whatever_the_input_order() {
        let attempts = vec![attempt(3, C, Some("/t/try-3")), attempt(1, A, Some("/t/try-1")), attempt(2, B, Some("/t/try-2"))];
        let kept = keep_plan("try/z/r/1", 1, &attempts, &[], &[(3, true), (2, true)]).unwrap();
        assert_eq!(kept.remove, vec![PathBuf::from("/t/try-2"), PathBuf::from("/t/try-3"), PathBuf::from("/t/try-1")]);
        assert_eq!(
            kept.delete,
            vec![
                ("refs/heads/try/z/r/1".to_string(), oid(A)),
                ("refs/heads/try/z/r/2".to_string(), oid(B)),
                ("refs/heads/try/z/r/3".to_string(), oid(C)),
            ]
        );
    }

    #[test]
    fn a_missing_attempt_is_refused() {
        match keep_plan("try/z/r/4", 4, &three(), &[], &[(1, true), (2, true), (3, true)]) {
            Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/4"),
            other => panic!("expected AttemptMissing, got {other:?}"),
        }
    }

    #[test]
    fn the_missing_attempt_is_refused_before_unsaved_ones() {
        match keep_plan("try/z/r/9", 9, &three(), &[], &[]) {
            Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/9"),
            other => panic!("expected AttemptMissing, got {other:?}"),
        }
    }

    #[test]
    fn unlisted_and_false_dropped_attempts_are_refused_in_k_order() {
        match keep_plan("try/z/r/2", 2, &three(), &[], &[(3, false)]) {
            Err(FolderError::AttemptsUnsaved { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3")])
            }
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn one_unsaved_dropped_attempt_is_refused_alone() {
        match keep_plan("try/z/r/1", 1, &three(), &[], &[(2, true), (3, false)]) {
            Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-3")]),
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn the_kept_attempt_needs_no_saved_entry() {
        let kept = keep_plan("try/z/r/3", 3, &three(), &[], &[(1, true), (2, true), (3, false)]).unwrap();
        assert_eq!(kept.tip, oid(C));
    }

    #[test]
    fn attempts_without_folders_are_skipped_from_remove_but_need_a_saved_entry() {
        let attempts = vec![attempt(1, A, None), attempt(2, B, None), attempt(3, C, Some("/t/try-3"))];
        let kept = keep_plan("try/z/r/2", 2, &attempts, &[], &[(1, true), (3, true)]).unwrap();
        assert_eq!(
            kept,
            Kept {
                tip: oid(B),
                remove: vec![PathBuf::from("/t/try-3")],
                delete: vec![
                    ("refs/heads/try/z/r/1".to_string(), oid(A)),
                    ("refs/heads/try/z/r/2".to_string(), oid(B)),
                    ("refs/heads/try/z/r/3".to_string(), oid(C)),
                ],
            }
        );
    }

    #[test]
    fn a_folderless_dropped_attempt_with_no_entry_is_refused_naming_its_branch() {
        let attempts = vec![attempt(1, A, None), attempt(2, B, None), attempt(3, C, Some("/t/try-3"))];
        match keep_plan("try/z/r/2", 2, &attempts, &[], &[]) {
            Err(FolderError::AttemptsUnsaved { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("refs/heads/try/z/r/1"), PathBuf::from("/t/try-3")])
            }
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn a_lone_attempt_is_kept_with_nothing_dropped() {
        let kept = keep_plan("try/z/r/1", 1, &[attempt(1, A, Some("/t/try-1"))], &[], &[]).unwrap();
        assert_eq!(
            kept,
            Kept {
                tip: oid(A),
                remove: vec![PathBuf::from("/t/try-1")],
                delete: vec![("refs/heads/try/z/r/1".to_string(), oid(A))],
            }
        );
    }

    #[test]
    fn a_saved_orphan_is_still_refused_naming_its_folder() {
        let orphans = vec![(4, PathBuf::from("/t/try-4"))];
        match keep_plan("try/z/r/2", 2, &three(), &orphans, &[(1, true), (3, true), (4, true)]) {
            Err(FolderError::OrphanAttempt { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-4")]),
            other => panic!("expected OrphanAttempt, got {other:?}"),
        }
    }

    #[test]
    fn orphans_are_refused_before_unsaved_attempts_in_k_order() {
        let orphans = vec![(5, PathBuf::from("/t/try-5")), (4, PathBuf::from("/t/try-4"))];
        match keep_plan("try/z/r/2", 2, &three(), &orphans, &[]) {
            Err(FolderError::OrphanAttempt { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("/t/try-4"), PathBuf::from("/t/try-5")])
            }
            other => panic!("expected OrphanAttempt, got {other:?}"),
        }
    }

    #[test]
    fn keeping_an_orphan_k_is_refused_as_missing_not_as_an_orphan() {
        let orphans = vec![(4, PathBuf::from("/t/try-4")), (5, PathBuf::from("/t/try-5"))];
        match keep_plan("try/z/r/4", 4, &three(), &orphans, &[(1, true), (2, true), (3, true)]) {
            Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/4"),
            other => panic!("expected AttemptMissing, got {other:?}"),
        }
    }
}

mod attempts_remaining {
    //! attempts_remaining: a request's try branches, then its attempt folders whose branch is not among them; refs and
    //! branches whose tail is not a canonical attempt number, other requests, zones, req/ branches and detached
    //! worktrees not counted.

    use std::path::PathBuf;

    use gitio::{Oid, Worktree};

    use folders::attempts::attempts_remaining;

    const A: &str = "0123456789abcdef0123456789abcdef01234567";

    fn refs(names: &[&str]) -> Vec<(Oid, String)> {
        names.iter().map(|name| (Oid::parse(A).unwrap(), name.to_string())).collect()
    }

    fn worktree(path: &str, branch: Option<&str>) -> Worktree {
        Worktree {
            path: PathBuf::from(path),
            head: Some(Oid::parse(A).unwrap()),
            branch: branch.map(str::to_string),
            bare: false,
            detached: branch.is_none(),
            locked: false,
            prunable: false,
        }
    }

    #[test]
    fn nothing_remains_with_no_refs_and_no_worktrees() {
        assert_eq!(attempts_remaining("z", "r", &[], &[]), Vec::<String>::new());
    }

    #[test]
    fn try_branches_are_named_in_input_order() {
        let input = refs(&["refs/heads/try/z/r/2", "refs/heads/try/z/r/1"]);
        assert_eq!(
            attempts_remaining("z", "r", &input, &[]),
            vec!["refs/heads/try/z/r/2".to_string(), "refs/heads/try/z/r/1".to_string()]
        );
    }

    #[test]
    fn a_folder_whose_branch_is_listed_is_not_named_again() {
        let input = refs(&["refs/heads/try/z/r/1"]);
        let trees = [worktree("/wt/try-1", Some("refs/heads/try/z/r/1"))];
        assert_eq!(attempts_remaining("z", "r", &input, &trees), vec!["refs/heads/try/z/r/1".to_string()]);
    }

    #[test]
    fn folders_whose_branch_is_not_listed_follow_the_branches_in_input_order() {
        let input = refs(&["refs/heads/try/z/r/1"]);
        let trees = [
            worktree("/wt/main", Some("refs/heads/main")),
            worktree("/wt/try-3", Some("refs/heads/try/z/r/3")),
            worktree("/wt/try-1", Some("refs/heads/try/z/r/1")),
            worktree("/wt/try-2", Some("refs/heads/try/z/r/2")),
        ];
        assert_eq!(
            attempts_remaining("z", "r", &input, &trees),
            vec!["refs/heads/try/z/r/1".to_string(), "/wt/try-3".to_string(), "/wt/try-2".to_string()]
        );
    }

    #[test]
    fn a_request_sharing_a_prefix_is_not_counted() {
        let input = refs(&["refs/heads/try/z/rx/1"]);
        let trees = [worktree("/wt/rx-2", Some("refs/heads/try/z/rx/2"))];
        assert_eq!(attempts_remaining("z", "r", &input, &trees), Vec::<String>::new());
    }

    #[test]
    fn another_zone_is_not_counted() {
        let input = refs(&["refs/heads/try/y/r/1"]);
        let trees = [worktree("/wt/y-2", Some("refs/heads/try/y/r/2"))];
        assert_eq!(attempts_remaining("z", "r", &input, &trees), Vec::<String>::new());
    }

    #[test]
    fn request_branches_and_detached_worktrees_are_not_counted() {
        let input = refs(&["refs/heads/req/z/r", "refs/heads/try/z/r"]);
        let trees = [worktree("/wt/req", Some("refs/heads/req/z/r")), worktree("/wt/detached", None)];
        assert_eq!(attempts_remaining("z", "r", &input, &trees), Vec::<String>::new());
    }

    #[test]
    fn a_tail_that_is_not_a_canonical_attempt_number_is_not_counted() {
        let input = refs(&["refs/heads/try/z/r/01", "refs/heads/try/z/r/0", "refs/heads/try/z/r/1/x"]);
        assert_eq!(attempts_remaining("z", "r", &input, &[]), Vec::<String>::new());
    }

    #[test]
    fn a_folder_on_a_branch_with_a_leading_zero_is_not_counted() {
        let trees = [worktree("/wt/try-01", Some("refs/heads/try/z/r/01"))];
        assert_eq!(attempts_remaining("z", "r", &[], &trees), Vec::<String>::new());
    }

    #[test]
    fn only_the_canonical_attempts_among_hand_made_refs_are_counted() {
        let input = refs(&["refs/heads/try/z/r/01", "refs/heads/try/z/r/3", "refs/heads/try/z/r/3/x"]);
        let trees = [
            worktree("/wt/try-0", Some("refs/heads/try/z/r/0")),
            worktree("/wt/try-4", Some("refs/heads/try/z/r/4")),
        ];
        assert_eq!(
            attempts_remaining("z", "r", &input, &trees),
            vec!["refs/heads/try/z/r/3".to_string(), "/wt/try-4".to_string()]
        );
    }
}

mod orphan_attempts {
    //! orphan_attempts: attempt folders whose branch is not among the refs, as (k, path) in k order; a present branch,
    //! a non-canonical tail, another request or zone and a detached worktree not listed.

    use std::path::PathBuf;

    use gitio::{Oid, Worktree};

    use folders::attempts::orphan_attempts;

    const A: &str = "0123456789abcdef0123456789abcdef01234567";

    fn refs(names: &[&str]) -> Vec<(Oid, String)> {
        names.iter().map(|name| (Oid::parse(A).unwrap(), name.to_string())).collect()
    }

    fn worktree(path: &str, branch: Option<&str>) -> Worktree {
        Worktree {
            path: PathBuf::from(path),
            head: Some(Oid::parse(A).unwrap()),
            branch: branch.map(str::to_string),
            bare: false,
            detached: branch.is_none(),
            locked: false,
            prunable: false,
        }
    }

    #[test]
    fn a_folder_whose_branch_is_gone_is_listed_with_its_k() {
        let trees = [worktree("/wt/try-2", Some("refs/heads/try/z/r/2"))];
        assert_eq!(orphan_attempts("z", "r", &[], &trees), vec![(2, PathBuf::from("/wt/try-2"))]);
    }

    #[test]
    fn a_folder_whose_branch_is_present_is_not_listed() {
        let input = refs(&["refs/heads/try/z/r/1"]);
        let trees = [worktree("/wt/try-1", Some("refs/heads/try/z/r/1"))];
        assert_eq!(orphan_attempts("z", "r", &input, &trees), Vec::<(u32, PathBuf)>::new());
    }

    #[test]
    fn a_leading_zero_tail_is_not_listed() {
        let trees = [worktree("/wt/try-01", Some("refs/heads/try/z/r/01"))];
        assert_eq!(orphan_attempts("z", "r", &[], &trees), Vec::<(u32, PathBuf)>::new());
    }

    #[test]
    fn another_request_or_zone_is_not_listed() {
        let trees = [
            worktree("/wt/rx-1", Some("refs/heads/try/z/rx/1")),
            worktree("/wt/y-1", Some("refs/heads/try/y/r/1")),
        ];
        assert_eq!(orphan_attempts("z", "r", &[], &trees), Vec::<(u32, PathBuf)>::new());
    }

    #[test]
    fn a_detached_worktree_is_not_listed() {
        let trees = [worktree("/wt/detached", None)];
        assert_eq!(orphan_attempts("z", "r", &[], &trees), Vec::<(u32, PathBuf)>::new());
    }

    #[test]
    fn orphans_are_listed_in_k_order_whatever_the_input_order() {
        let input = refs(&["refs/heads/try/z/r/2"]);
        let trees = [
            worktree("/wt/try-10", Some("refs/heads/try/z/r/10")),
            worktree("/wt/try-2", Some("refs/heads/try/z/r/2")),
            worktree("/wt/try-3", Some("refs/heads/try/z/r/3")),
            worktree("/wt/try-1", Some("refs/heads/try/z/r/1")),
        ];
        assert_eq!(
            orphan_attempts("z", "r", &input, &trees),
            vec![(1, PathBuf::from("/wt/try-1")), (3, PathBuf::from("/wt/try-3")), (10, PathBuf::from("/wt/try-10"))]
        );
    }
}

mod attempts_of {
    //! attempts_of: a request's try branches, sorted by k, each with the folder that has it checked out; other refs
    //! and odd attempt numbers ignored.

    use std::path::PathBuf;

    use gitio::Worktree;

    use folders::attempts::{attempts_of, Attempt};

    use crate::ref_pairs::{oid, refs};

    const A: &str = "0123456789abcdef0123456789abcdef01234567";
    const B: &str = "fedcba9876543210fedcba9876543210fedcba98";
    const C: &str = "1111111111111111111111111111111111111111";

    fn worktree(path: &str, branch: Option<&str>) -> Worktree {
        Worktree {
            path: PathBuf::from(path),
            head: Some(oid(A)),
            branch: branch.map(str::to_string),
            bare: false,
            detached: branch.is_none(),
            locked: false,
            prunable: false,
        }
    }

    #[test]
    fn reads_attempts_sorted_by_k_with_their_folders() {
        let input = refs(&[
            (C, "refs/heads/try/z/r/10"),
            (A, "refs/heads/req/z/r"),
            (B, "refs/heads/try/z/r/2"),
            (A, "refs/heads/try/z/r/1"),
        ]);
        let trees = [
            worktree("/w/r", Some("refs/heads/req/z/r")),
            worktree("/w/tries/try-2", Some("refs/heads/try/z/r/2")),
            worktree("/w/tries/try-10", Some("refs/heads/try/z/r/10")),
            worktree("/w/detached", None),
        ];
        assert_eq!(
            attempts_of("z", "r", &input, &trees),
            vec![
                Attempt { k: 1, branch: "refs/heads/try/z/r/1".to_string(), tip: oid(A), folder: None },
                Attempt {
                    k: 2,
                    branch: "refs/heads/try/z/r/2".to_string(),
                    tip: oid(B),
                    folder: Some(PathBuf::from("/w/tries/try-2")),
                },
                Attempt {
                    k: 10,
                    branch: "refs/heads/try/z/r/10".to_string(),
                    tip: oid(C),
                    folder: Some(PathBuf::from("/w/tries/try-10")),
                },
            ]
        );
    }

    #[test]
    fn ignores_other_requests_zones_and_shared_prefixes() {
        let input = refs(&[
            (A, "refs/heads/try/z/other/1"),
            (A, "refs/heads/try/y/r/1"),
            (A, "refs/heads/try/z/rx/1"),
            (A, "refs/heads/try/z/r"),
            (A, "refs/mortar/trysnap/z/r/1"),
            (A, "refs/heads/req/z/r/1"),
        ]);
        assert_eq!(attempts_of("z", "r", &input, &[]), Vec::<Attempt>::new());
    }

    #[test]
    fn ignores_odd_attempt_numbers() {
        let input = refs(&[
            (A, "refs/heads/try/z/r/0"),
            (A, "refs/heads/try/z/r/01"),
            (A, "refs/heads/try/z/r/"),
            (A, "refs/heads/try/z/r/+1"),
            (A, "refs/heads/try/z/r/-1"),
            (A, "refs/heads/try/z/r/1x"),
            (A, "refs/heads/try/z/r/1/2"),
            (A, "refs/heads/try/z/r/4294967296"),
        ]);
        assert_eq!(attempts_of("z", "r", &input, &[]), Vec::<Attempt>::new());
    }

    #[test]
    fn reads_the_largest_attempt_number() {
        let input = refs(&[(B, "refs/heads/try/z/r/4294967295")]);
        assert_eq!(
            attempts_of("z", "r", &input, &[]),
            vec![Attempt { k: 4294967295, branch: "refs/heads/try/z/r/4294967295".to_string(), tip: oid(B), folder: None }]
        );
    }
}

mod closed_snap_refs {
    //! closed_snap_refs: a closed request's snapshot and attempt snapshot refs are picked; a request branch or an
    //! attempt branch keeps them open, by exact name; a non-canonical branch under the try prefix does not; odd-shaped
    //! snapshot refs and other refs are ignored; input order is kept.

    use gitio::Oid;

    use folders::sweep::closed_snap_refs;

    use crate::ref_pairs::{oid, refs};

    const A: &str = "0123456789abcdef0123456789abcdef01234567";
    const B: &str = "fedcba9876543210fedcba9876543210fedcba98";

    #[test]
    fn an_open_request_is_kept() {
        let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/mortar/snap/z/r")]);
        assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
    }

    #[test]
    fn a_closed_request_is_picked() {
        let input = refs(&[(A, "refs/heads/zone/z"), (B, "refs/mortar/snap/z/r")]);
        assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
    }

    #[test]
    fn an_attempt_branch_keeps_it_open() {
        let input = refs(&[(B, "refs/mortar/snap/z/r"), (A, "refs/heads/try/z/r/1")]);
        assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
    }

    #[test]
    fn a_closed_requests_trysnap_ref_is_picked() {
        let input = refs(&[
            (A, "refs/heads/zone/z"),
            (B, "refs/mortar/trysnap/z/r/1"),
            (A, "refs/mortar/snap/z/r"),
            (A, "refs/mortar/trysnap/z/r/2"),
        ]);
        assert_eq!(
            closed_snap_refs(&input),
            vec![
                ("refs/mortar/trysnap/z/r/1".to_string(), oid(B)),
                ("refs/mortar/snap/z/r".to_string(), oid(A)),
                ("refs/mortar/trysnap/z/r/2".to_string(), oid(A)),
            ]
        );
    }

    #[test]
    fn a_request_branch_keeps_snap_and_trysnap_refs() {
        let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/mortar/snap/z/r"), (B, "refs/mortar/trysnap/z/r/1")]);
        assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
    }

    #[test]
    fn a_try_branch_keeps_snap_and_trysnap_refs() {
        let input = refs(&[(B, "refs/mortar/snap/z/r"), (B, "refs/mortar/trysnap/z/r/1"), (A, "refs/heads/try/z/r/2")]);
        assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
    }

    fn picked_beside(branch: &str) -> Vec<(String, Oid)> {
        closed_snap_refs(&refs(&[(A, branch), (B, "refs/mortar/snap/z/r"), (A, "refs/mortar/trysnap/z/r/1")]))
    }

    #[test]
    fn a_try_branch_with_a_leading_zero_does_not_keep_it_open() {
        assert_eq!(
            picked_beside("refs/heads/try/z/r/01"),
            vec![("refs/mortar/snap/z/r".to_string(), oid(B)), ("refs/mortar/trysnap/z/r/1".to_string(), oid(A))]
        );
    }

    #[test]
    fn a_try_branch_numbered_zero_does_not_keep_it_open() {
        assert_eq!(
            picked_beside("refs/heads/try/z/r/0"),
            vec![("refs/mortar/snap/z/r".to_string(), oid(B)), ("refs/mortar/trysnap/z/r/1".to_string(), oid(A))]
        );
    }

    #[test]
    fn a_try_branch_nested_under_an_attempt_does_not_keep_it_open() {
        assert_eq!(
            picked_beside("refs/heads/try/z/r/1/x"),
            vec![("refs/mortar/snap/z/r".to_string(), oid(B)), ("refs/mortar/trysnap/z/r/1".to_string(), oid(A))]
        );
    }

    #[test]
    fn a_try_branch_of_a_request_sharing_a_prefix_does_not_keep_it_open() {
        let input = refs(&[(A, "refs/heads/try/z/rx/1"), (B, "refs/mortar/trysnap/z/r/1"), (A, "refs/mortar/snap/z/r")]);
        assert_eq!(
            closed_snap_refs(&input),
            vec![("refs/mortar/trysnap/z/r/1".to_string(), oid(B)), ("refs/mortar/snap/z/r".to_string(), oid(A))]
        );
    }

    #[test]
    fn a_ref_nested_under_the_request_branch_name_does_not_keep_it_open() {
        let input = refs(&[(A, "refs/heads/req/z/r/try-1"), (B, "refs/mortar/snap/z/r")]);
        assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
    }

    #[test]
    fn an_odd_shaped_trysnap_ref_is_ignored() {
        let input = refs(&[
            (A, "refs/mortar/trysnap/z/r"),
            (A, "refs/mortar/trysnap/z/r/"),
            (A, "refs/mortar/trysnap/z//1"),
            (A, "refs/mortar/trysnap//r/1"),
            (A, "refs/mortar/trysnap/z/r/1/extra"),
            (A, "refs/mortar/trysnap/z"),
        ]);
        assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
    }

    #[test]
    fn a_branch_sharing_a_prefix_does_not_keep_it_open() {
        let input = refs(&[(A, "refs/heads/req/z/rx"), (B, "refs/mortar/snap/z/r")]);
        assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
    }

    #[test]
    fn a_same_request_in_another_zone_does_not_keep_it_open() {
        let input = refs(&[(A, "refs/heads/req/y/r"), (B, "refs/mortar/snap/z/r")]);
        assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
    }

    #[test]
    fn an_odd_shaped_snap_ref_is_ignored() {
        let input = refs(&[
            (A, "refs/mortar/snap/z"),
            (A, "refs/mortar/snap/z/r/extra"),
            (A, "refs/mortar/snap//r"),
            (A, "refs/mortar/snap/z/"),
        ]);
        assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
    }

    #[test]
    fn a_non_snap_ref_is_ignored() {
        let input = refs(&[(A, "refs/heads/main"), (B, "refs/mortar/other/z/r")]);
        assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
    }

    #[test]
    fn input_order_is_kept() {
        let input = refs(&[
            (B, "refs/mortar/snap/z/b"),
            (A, "refs/heads/req/z/open"),
            (A, "refs/mortar/snap/z/open"),
            (A, "refs/mortar/snap/y/a"),
        ]);
        assert_eq!(
            closed_snap_refs(&input),
            vec![("refs/mortar/snap/z/b".to_string(), oid(B)), ("refs/mortar/snap/y/a".to_string(), oid(A))]
        );
    }
}
