use gitio::Git;

/// gitio-specific adapter; shared testkit owns initialization, path and temporary-directory cleanup.
pub struct GitRepo {
    pub git: Git,
    pub(super) repository: testkit::TestRepo,
}
