use git2::{
    build::{CheckoutBuilder, CloneLocal, RepoBuilder},
    Repository,
};
use std::path::Path;
use tempfile::TempDir;

pub fn repo_init() -> (TempDir, Repository) {
    let td = TempDir::new().unwrap();
    let repo = Repository::init(td.path()).unwrap();
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "name").unwrap();
        config.set_str("user.email", "email").unwrap();
        let mut index = repo.index().unwrap();
        let id = index.write_tree().unwrap();

        let tree = repo.find_tree(id).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }
    (td, repo)
}

pub fn repo_clone(from: &Path) -> (TempDir, Repository) {
    let td = TempDir::new().unwrap();

    let co = CheckoutBuilder::new();

    let repo = RepoBuilder::new()
        .with_checkout(co)
        .clone_local(CloneLocal::Local)
        .remote_create(|repo, _name, url| repo.remote("origin", url))
        .clone(from.to_str().unwrap(), td.path())
        .unwrap();
    (td, repo)
}
