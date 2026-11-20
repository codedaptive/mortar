use super::manifest_repos;

#[test]
fn quoted_repo_lines_keep_order_and_duplicates() {
    let text = "repo = \"/one\"\n  repo=\"/two\" # comment\n\trepo  =  \"/one\"\nrepo = \"\"\n";
    assert_eq!(manifest_repos(text), vec!["/one", "/two", "/one", ""]);
}

#[test]
fn non_repo_keys_comments_unquoted_and_unclosed_values_are_skipped() {
    let text = "# repo = \"/comment\"\n  # repo = \"/comment2\"\nrepository = \"/wrong\"\nrepo_url = \"/wrong\"\nrepo = unquoted\nrepo = \"unclosed\nother = \"repo = x\"\n";
    assert_eq!(manifest_repos(text), Vec::<String>::new());
}

#[test]
fn the_value_ends_at_the_next_quote() {
    assert_eq!(manifest_repos("repo = \"first\" extra \"second\"\n"), vec!["first"]);
}
