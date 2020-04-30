use crate::git::{Git, GitCommand};
use git2::{
    build::{CheckoutBuilder, CloneLocal, RepoBuilder},
    BranchType, Repository,
};
use std::cell::RefCell;
use std::path::Path;
use tempfile::TempDir;

pub fn new_repo() -> (TempDir, Repository) {
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

pub fn new_bare_repo() -> (TempDir, Repository) {
    let td = TempDir::new().unwrap();
    let repo = Repository::init_bare(td.path()).unwrap();
    (td, repo)
}

pub fn clone_repo(from: &Path) -> (TempDir, Repository) {
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

pub fn new_git<'repo>() -> ((TempDir, TempDir), GitCommand<'repo>) {
    let (origin_dir, _) = crate::test::new_repo();
    let (clone_dir, clone_repo) = crate::test::clone_repo(origin_dir.path());
    ((origin_dir, clone_dir), GitCommand::from_repo(clone_repo))
}

pub struct MockGit {
    pub commands: RefCell<Vec<String>>,
}

impl MockGit {
    pub fn new() -> Self {
        Self {
            commands: RefCell::new(vec![]),
        }
    }
}

impl Git for MockGit {
    fn run(&self, args: &[&str]) -> anyhow::Result<()> {
        self.commands.borrow_mut().push(args.join(" ").to_string());
        Ok(())
    }
    fn tree_is_clean(&self) -> anyhow::Result<bool> {
        unimplemented!()
    }
    fn has_branch(&self, _branch: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
    fn on_branch(&self, branch: &str) -> anyhow::Result<bool> {
        todo!()
    }
}
