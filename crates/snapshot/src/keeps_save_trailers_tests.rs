use crate::keeps_save_trailers::keeps_save_trailers;

type Pairs = Vec<(String, String)>;

fn pairs(rows: &[(&str, &str)]) -> Pairs {
    rows.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect()
}

#[test]
fn keeps_save_trailers_is_true_only_for_exactly_the_wanted_pairs_in_order() {
    let id = ("Mortar-Change-Id", "kkkkllllmmmmnnnnooooppppqqqqrrrr");
    let request = ("Mortar-Request", "r3");
    let zone = ("Mortar-Zone", "snapshot");
    let wanted = pairs(&[id, request, zone]);
    let rows: Vec<(&str, Pairs, bool)> = vec![
        ("exact match", pairs(&[id, request, zone]), true),
        ("a Signed-off-by after", pairs(&[id, request, zone, ("Signed-off-by", "A <a@x>")]), true),
        (
            "lower-case keys, same values",
            pairs(&[
                ("mortar-change-id", "kkkkllllmmmmnnnnooooppppqqqqrrrr"),
                ("mortar-request", "r3"),
                ("mortar-zone", "snapshot"),
            ]),
            true,
        ),
        ("no pairs", pairs(&[]), false),
        ("Mortar-Zone missing", pairs(&[id, request]), false),
        (
            "Mortar-Change-Id with another value",
            pairs(&[("Mortar-Change-Id", "aaaabbbbccccddddeeeeffffgggghhhh"), request, zone]),
            false,
        ),
        ("the set repeated twice", pairs(&[id, request, zone, id, request, zone]), false),
        ("order swapped", pairs(&[request, id, zone]), false),
    ];
    for (case, given, expected) in rows {
        assert_eq!(keeps_save_trailers(&given, &wanted), expected, "{case}");
    }
}
