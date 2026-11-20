use crate::fixture::repo;

#[test]
fn mortar_trailers_written_last_survive_a_body_divider() {
    // Joins Git::with_trailers -> real interpret-trailers -> Git::trailers.
    let r = repo();
    let body = "Subject\n\nForeign: above-divider\n\n---\nbody after divider\n";
    let expected = vec![
        ("Mortar-Change-Id".into(), "klmnopqrstuvwxyzklmnopqrstuvwxyz".into()),
        ("Mortar-Request".into(), "r1".into()),
        ("Mortar-Zone".into(), "gitio".into()),
    ];
    let message = r.git.with_trailers(body, &expected).unwrap();
    assert!(message.contains("---\nbody after divider"), "{message}");
    assert_eq!(r.git.trailers(&message).unwrap(), expected);
    // Foreign trailers above a later divider are outside the final trailer block.
    assert!(r.git.trailers(body).unwrap().is_empty());
}

#[test]
fn trailers_preserve_order_and_unfold_continuation_lines() {
    // Joins Git::trailers -> real git trailer grammar -> parsed key/value pairs.
    let r = repo();
    let message = "Subject\n\nKey: first\n continuation\nMortar-Request: r1\nKey: second\n";
    assert_eq!(r.git.trailers(message).unwrap(), vec![
        ("Key".into(), "first continuation".into()),
        ("Mortar-Request".into(), "r1".into()),
        ("Key".into(), "second".into()),
    ]);
}
