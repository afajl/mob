use crate::command;
use crate::os;
use anyhow::{Context, Result};
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
        if self.dry_run {
            return Ok(());
        }
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

impl Git {
    pub fn save_meta<T: ?Sized>(&self, name: &str, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let json = serde_json::to_vec(data)?;
        let oid = self.repo.blob(json.as_slice())?;
        let mut tree = self.repo.treebuilder(None)?;
        tree.insert(name, oid, 0o100644)?;
        let tree = tree.write()?;
        let tree = self.repo.find_tree(tree)?;
        let signature = Git::get_signature()?;

        let parent = self.get_latest();
        let parent = match parent {
            Some(ref commit) => vec![commit],
            None => vec![],
        };

        self.repo.commit(
            Some("refs/heads/mob-meta"),
            &signature,
            &signature,
            "save",
            &tree,
            parent.as_slice(),
        )?;
        Ok(())
    }

    pub fn load_meta<T>(&self, name: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let commit = self.get_latest();

        let commit = match commit {
            Some(commit) => commit,
            None => return Ok(None),
        };

        let tree = commit.tree()?;

        let tree_entry = tree.get_name(name);

        let tree_entry = match tree_entry {
            Some(tree) => tree,
            None => return Ok(None),
        };

        let object = tree_entry.to_object(&self.repo)?;
        let blob = object.as_blob();
        let blob = match blob {
            Some(blob) => blob,
            None => return Ok(None),
        };
        let blob = blob.content();

        serde_json::from_slice(blob).context("failed to deserialize")
    }

    fn get_latest(&self) -> Option<Commit> {
        self.repo
            .find_reference("refs/heads/mob-meta")
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

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TestData {
        v: String,
    }

    #[test]
    fn create_and_read_meta() {
        let (_td, repo) = crate::test::repo_init();
        let git = Git::from_repo(repo);
        let data = "data".to_string();
        git.save_meta("meta", &data).expect("could not save data");
        let got: String = git.load_meta("meta").expect("could not load data").unwrap();
        assert_eq!(got, data)
    }

    #[test]
    fn missing_meta_ok() {
        let (_td, repo) = crate::test::repo_init();
        let git = Git::from_repo(repo);
        let got: Option<String> = git.load_meta("meta").expect("could not load data");
        assert!(got.is_none())
    }

    #[test]
    fn corrupt_meta_is_error() {
        let (_td, repo) = crate::test::repo_init();
        let git = Git::from_repo(repo);

        let data = vec![1, 2, 3];
        git.save_meta("meta", &data).expect("could not save data");
        let res: Result<Option<String>> = git.load_meta("meta");
        assert!(res.is_err())
    }

    #[test]
    fn latest_meta_loaded() {
        let (_td, repo) = crate::test::repo_init();
        let git = Git::from_repo(repo);
        let data = TestData {
            v: "first".to_string(),
        };
        git.save_meta("meta", &data).expect("could not save data");

        let data = TestData {
            v: "second".to_string(),
        };
        git.save_meta("meta", &data).expect("could not save data");

        let got: TestData = git.load_meta("meta").expect("could not load data").unwrap();
        assert_eq!(got.v, data.v)
    }
}
