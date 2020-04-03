use crate::command;
use crate::os;
use crate::store;
use crate::store::Store;
use anyhow::Result;
use git2::{Commit, Config, Error, Oid, Repository, Signature};
use log;
use serde::{de::DeserializeOwned, Serialize};
use std::env;
use std::path::PathBuf;

pub struct Git {
    command: command::Command<'static>,
    repo: Repository,
    dry_run: bool,
}

impl Git {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or(env::current_dir()?);
        let repo = Repository::open(&path)?;
        let command = command::Command::new(os::command("git")).working_directory(path.as_path());
        Ok(Self {
            command,
            repo,
            dry_run: false,
        })
    }

    pub fn from_repo(repo: Repository) -> Self {
        let command = command::Command::new(os::command("git")).working_directory(repo.path());
        Self {
            command,
            repo,
            dry_run: false,
        }
    }

    pub fn run(&self, args: &[&str]) -> Result<()> {
        log::debug!("run: git {}", args.join(" "));
        println!("run: git {}", args.join(" "));
        // if self.dry_run {
        //     return Ok(());
        // }
        self.command.run_checked(args)
    }

    pub fn tree_is_clean(&self) -> Result<bool> {
        if self.dry_run {
            return Ok(true);
        }
        self.command
            .run_stdout(&["status", "--short"])
            .map(|output| output.trim().len() == 0)
    }

    pub fn root_dir(&self) -> Result<PathBuf> {
        self.command
            .run_stdout(&["rev-parse", "--show-toplevel"])
            .map(|output| PathBuf::from(output.trim()))
    }

    pub fn on_branch(&self, branch: &str) -> Result<bool> {
        self.command
            .run_stdout(&["rev-parse", "--abbrev-ref", "HEAD"])
            .map(|output| output.contains(branch))
    }

    pub fn has_branch(&self, branch: &str) -> Result<bool> {
        self.command
            .run_stdout(&["rev-parse", "--quiet", "--verify", branch])
            .map(|output| output.trim().len() != 0)
    }
}

pub const MOB_META_BRANCH: &str = "mob-meta";

impl Store for Git {
    fn save<T: ?Sized>(&self, name: &str, data: &T) -> Result<(), store::Error>
    where
        T: Serialize,
    {
        let json = serde_json::to_vec(data)?;

        let oid = self.repo.blob(json.as_slice())?;

        let mut tree = self.repo.treebuilder(None)?;
        tree.insert(name, oid, 0o100644)?;
        let tree = tree.write()?;
        let tree = self.repo.find_tree(tree)?;

        let parent = self.get_latest();
        let parent = match parent {
            Some(ref commit) => vec![commit],
            None => vec![],
        };

        let signature = Git::get_signature()?;

        self.repo.commit(
            Some(format!("refs/heads/{}", MOB_META_BRANCH).as_str()),
            &signature,
            &signature,
            "save",
            &tree,
            parent.as_slice(),
        )?;

        self.run(&[
            "push",
            "origin",
            format!("{}:{}", MOB_META_BRANCH, MOB_META_BRANCH).as_str(),
        ])
        .map_err(store::Error::Conflict)
    }

    fn load<T>(&self, name: &str) -> Result<T, store::Error>
    where
        T: DeserializeOwned,
    {
        let commit = self.get_latest();

        let commit = match commit {
            Some(commit) => commit,
            None => return Err(store::Error::Missing),
        };

        let tree = commit.tree()?;

        let tree_entry = tree.get_name(name);

        let tree_entry = match tree_entry {
            Some(tree) => tree,
            None => return Err(store::Error::Missing),
        };

        let object = tree_entry.to_object(&self.repo)?;
        let blob = object.as_blob();
        let blob = match blob {
            Some(blob) => blob,
            None => return Err(store::Error::Missing),
        };
        let blob = blob.content();

        serde_json::from_slice(blob).map_err(store::Error::Format)
    }
}

impl Git {
    fn get_latest(&self) -> Option<Commit> {
        self.repo
            .find_reference(format!("refs/heads/{}", MOB_META_BRANCH).as_str())
            .and_then(|reference| reference.resolve())
            .and_then(|reference| {
                self.repo
                    .find_commit(reference.target().unwrap_or(Oid::zero()))
            })
            .ok()
    }

    fn get_signature() -> Result<Signature<'static>, Error> {
        let config = Config::open_default()?;
        let name = config.get_string("user.name")?;
        let email = config.get_string("user.email")?;
        Signature::now(name.as_str(), email.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::panic;
    use tempfile::TempDir;

    fn get_repo() -> (TempDir, TempDir, Git) {
        let (origin_dir, _) = crate::test::repo_init();
        let (clone_dir, clone_repo) = crate::test::repo_clone(origin_dir.path());
        (origin_dir, clone_dir, Git::from_repo(clone_repo))
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TestData {
        v: String,
    }

    #[test]
    fn create_and_read_meta() {
        let (_od, _cd, git) = get_repo();
        let data = "data".to_string();
        git.save("meta", &data).expect("could not save data");
        let got = git.load::<String>("meta").expect("could not load data");
        assert_eq!(got, data)
    }

    #[test]
    fn missing_meta_ok() {
        let (_od, _cd, git) = get_repo();
        let err = git.load::<String>("meta").unwrap_err();
        match err {
            store::Error::Missing => {}
            _ => panic!("Expected missing"),
        }
    }

    #[test]
    fn corrupt_meta_is_error() {
        let (_od, _cd, git) = get_repo();

        let data = vec![1, 2, 3];
        git.save("meta", &data).expect("could not save data");

        let err = git.load::<String>("meta").unwrap_err();

        match err {
            store::Error::Format(_) => {}
            _ => panic!("Expected format error"),
        }
    }

    #[test]
    fn latest_meta_loaded() {
        let (_od, _cd, git) = get_repo();

        let data = TestData {
            v: "first".to_string(),
        };
        git.save("meta", &data).expect("could not save data");

        let data = TestData {
            v: "second".to_string(),
        };
        git.save("meta", &data).expect("could not save data");

        let got: TestData = git.load("meta").expect("could not load data");
        assert_eq!(got.v, data.v)
    }

    #[test]
    fn push_conflict() {
        let (origin, _) = crate::test::repo_init();
        let (_clone1, clonerepo1) = crate::test::repo_clone(origin.path());
        let (_clone2, clonerepo2) = crate::test::repo_clone(origin.path());

        let git1 = Git::from_repo(clonerepo1);
        let data = "first state".to_string();
        git1.save("test", &data).unwrap();

        let git2 = Git::from_repo(clonerepo2);
        let data = "second state".to_string();
        let err = git2.save("test", &data).unwrap_err();
        match err {
            store::Error::Conflict(_) => {}
            _ => panic!("Pushing to remote twice should result in conflict"),
        }
    }
}
