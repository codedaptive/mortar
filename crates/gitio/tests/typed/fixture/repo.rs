use crate::support::TestRepo;

pub fn repo() -> TestRepo {
    let mut repo = TestRepo::new();
    // Deterministic fixture commits; explicit Person arguments still override these defaults.
    repo.git.env.extend([
        ("GIT_AUTHOR_NAME".into(), "Fixture Author".into()),
        ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
        ("GIT_AUTHOR_DATE".into(), "2000-01-01T00:00:00 +0000".into()),
        ("GIT_COMMITTER_NAME".into(), "Fixture Committer".into()),
        ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
        ("GIT_COMMITTER_DATE".into(), "2000-01-02T00:00:00 +0000".into()),
    ]);
    repo
}
