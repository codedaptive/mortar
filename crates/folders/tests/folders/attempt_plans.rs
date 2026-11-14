//! Zone integration tests of drop_plan, keep_plan and attempts_remaining, with unsaved_attempts / orphan_attempts
//! used for real (S-79 as amended by #1264).

mod drop_plan {
    //! drop_plan: unsaved attempts (folder or not) refused in k order, then unsaved orphan folders; every attempt
    //! folder to remove in k order, then orphan folders; every branch to delete, in k order, none for orphans; no
    //! attempts and no orphans plan nothing.

    use std::path::PathBuf;

    use gitio::Oid;

    use folders::attempts::{drop_plan, Attempt, Dropped};
    use folders::FolderError;

    const A: &str = "1111111111111111111111111111111111111111";
    const B: &str = "2222222222222222222222222222222222222222";
    const C: &str = "3333333333333333333333333333333333333333";

    fn oid(hex: &str) -> Oid {
        Oid::parse(hex).unwrap()
    }

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
    fn an_unsaved_orphan_is_refused_naming_its_folder() {
        let attempts = vec![attempt(1, A, Some("/t/try-1"))];
        let orphans = vec![(2, PathBuf::from("/t/try-2")), (3, PathBuf::from("/t/try-3"))];
        match drop_plan(&attempts, &orphans, &[(1, true), (3, true), (2, false)]) {
            Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-2")]),
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn a_saved_orphan_is_removed_after_the_attempts_and_not_deleted() {
        let attempts = vec![attempt(3, C, Some("/t/try-3"))];
        let orphans = vec![(1, PathBuf::from("/t/try-1"))];
        let dropped = drop_plan(&attempts, &orphans, &[(1, true), (3, true)]).unwrap();
        assert_eq!(
            dropped,
            Dropped {
                remove: vec![PathBuf::from("/t/try-3"), PathBuf::from("/t/try-1")],
                delete: vec![("refs/heads/try/z/r/3".to_string(), oid(C))],
            }
        );
    }

    #[test]
    fn saved_orphans_alone_are_removed_in_k_order_with_nothing_deleted() {
        let orphans = vec![(4, PathBuf::from("/t/try-4")), (2, PathBuf::from("/t/try-2"))];
        let dropped = drop_plan(&[], &orphans, &[(2, true), (4, true)]).unwrap();
        assert_eq!(
            dropped,
            Dropped { remove: vec![PathBuf::from("/t/try-2"), PathBuf::from("/t/try-4")], delete: Vec::new() }
        );
    }
}

mod keep_plan {
    //! keep_plan: a missing attempt and unsaved dropped attempts (folder or not) refused; the kept tip, the folders to
    //! remove (the kept one last) and every branch to delete, in k order.

    use std::path::PathBuf;

    use gitio::Oid;

    use folders::attempts::{keep_plan, Attempt, Kept};
    use folders::FolderError;

    const A: &str = "1111111111111111111111111111111111111111";
    const B: &str = "2222222222222222222222222222222222222222";
    const C: &str = "3333333333333333333333333333333333333333";

    fn oid(hex: &str) -> Oid {
        Oid::parse(hex).unwrap()
    }

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
        let kept = keep_plan("try/z/r/2", 2, &three(), &[(1, true), (3, true)]).unwrap();
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
        let kept = keep_plan("try/z/r/1", 1, &attempts, &[(3, true), (2, true)]).unwrap();
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
        match keep_plan("try/z/r/4", 4, &three(), &[(1, true), (2, true), (3, true)]) {
            Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/4"),
            other => panic!("expected AttemptMissing, got {other:?}"),
        }
    }

    #[test]
    fn the_missing_attempt_is_refused_before_unsaved_ones() {
        match keep_plan("try/z/r/9", 9, &three(), &[]) {
            Err(FolderError::AttemptMissing { branch }) => assert_eq!(branch, "try/z/r/9"),
            other => panic!("expected AttemptMissing, got {other:?}"),
        }
    }

    #[test]
    fn unlisted_and_false_dropped_attempts_are_refused_in_k_order() {
        match keep_plan("try/z/r/2", 2, &three(), &[(3, false)]) {
            Err(FolderError::AttemptsUnsaved { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3")])
            }
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn one_unsaved_dropped_attempt_is_refused_alone() {
        match keep_plan("try/z/r/1", 1, &three(), &[(2, true), (3, false)]) {
            Err(FolderError::AttemptsUnsaved { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-3")]),
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn the_kept_attempt_needs_no_saved_entry() {
        let kept = keep_plan("try/z/r/3", 3, &three(), &[(1, true), (2, true), (3, false)]).unwrap();
        assert_eq!(kept.tip, oid(C));
    }

    #[test]
    fn attempts_without_folders_are_skipped_from_remove_but_need_a_saved_entry() {
        let attempts = vec![attempt(1, A, None), attempt(2, B, None), attempt(3, C, Some("/t/try-3"))];
        let kept = keep_plan("try/z/r/2", 2, &attempts, &[(1, true), (3, true)]).unwrap();
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
        match keep_plan("try/z/r/2", 2, &attempts, &[]) {
            Err(FolderError::AttemptsUnsaved { paths }) => {
                assert_eq!(paths, vec![PathBuf::from("refs/heads/try/z/r/1"), PathBuf::from("/t/try-3")])
            }
            other => panic!("expected AttemptsUnsaved, got {other:?}"),
        }
    }

    #[test]
    fn a_lone_attempt_is_kept_with_nothing_dropped() {
        let kept = keep_plan("try/z/r/1", 1, &[attempt(1, A, Some("/t/try-1"))], &[]).unwrap();
        assert_eq!(
            kept,
            Kept {
                tip: oid(A),
                remove: vec![PathBuf::from("/t/try-1")],
                delete: vec![("refs/heads/try/z/r/1".to_string(), oid(A))],
            }
        );
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
