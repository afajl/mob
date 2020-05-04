pub mod store;
use crate::command;
use crate::os;
use anyhow::Result;
pub use git2::BranchType;
use git2::{Commit, Config, Error, Oid, Repository, Signature};
use std::env;
use std::path::PathBuf;
pub use store::Store;

pub trait Git {
    fn run(&self, args: &[&str]) -> Result<()>;
    fn tree_is_clean(&self) -> Result<bool>;
    fn has_branch(&self, branch: &str) -> Result<bool>;
    fn on_branch(&self, branch: &str) -> Result<bool>;
}

#[derive(Debug)]
pub struct CommitFile<'a> {
    pub filename: &'a str,
    pub data: &'a [u8],
    pub message: &'a str,
    pub reference: &'a str,
}

pub struct GitCommand<'repo> {
    command: command::Command<'repo>,
    repo: Repository,
    pub remote: String,
}

impl<'repo> GitCommand<'repo> {
    pub fn new(path: Option<PathBuf>, remote: String) -> Result<GitCommand<'repo>> {
        let path = path.unwrap_or(env::current_dir()?);
        let repo = Repository::open(&path)?;
        let command = command::Command::new(os::command("git")).working_directory(path.as_path());
        Ok(Self {
            command,
            repo,
            remote,
        })
    }

    pub fn from_repo(repo: Repository) -> Self {
        let workdir = repo.workdir().expect("Repo does not have a workdir");
        let command = command::Command::new(os::command("git")).working_directory(workdir);
        Self {
            command,
            repo,
            remote: "origin".into(),
        }
    }

    fn last_commit(&self, reference: &str) -> Option<Commit> {
        let absolute_ref = format!("refs/heads/{}", reference);

        self.repo
            .find_reference(absolute_ref.as_str())
            .and_then(|reference| reference.resolve())
            .and_then(|reference| {
                self.repo
                    .find_commit(reference.target().unwrap_or_else(Oid::zero))
            })
            .ok()
    }

    fn get_signature() -> Result<Signature<'static>, Error> {
        let config = Config::open_default()?;
        let name = config.get_string("user.name")?;
        let email = config.get_string("user.email")?;
        Signature::now(name.as_str(), email.as_str())
    }

    pub fn create_commit<'a>(&self, commit: CommitFile<'a>) -> Result<git2::Oid, Error> {
        let oid = self.repo.blob(commit.data)?;

        let mut tree = self.repo.treebuilder(None)?;
        tree.insert(commit.filename, oid, 0o100_644)?;
        let tree = tree.write()?;
        let tree = self.repo.find_tree(tree)?;

        let parent = self.last_commit(commit.reference);
        let parent = match parent {
            Some(ref commit) => vec![commit],
            None => vec![],
        };

        let signature = GitCommand::get_signature()?;

        let absolute_ref = format!("refs/heads/{}", commit.reference);
        self.repo.commit(
            Some(absolute_ref.as_str()),
            &signature,
            &signature,
            commit.message,
            &tree,
            parent.as_slice(),
        )
    }

    fn find_branch(&self, name: &str) -> Result<Option<git2::Branch>> {
        let branches = self.repo.branches(None)?;

        for branch_result in branches {
            let branch = branch_result?.0;

            if let Ok(Some(branch_name)) = branch.name() {
                if branch_name == name {
                    return Ok(Some(branch));
                }
            }
        }
        Ok(None)
    }

    fn run_quietly(&self, args: &[&str]) -> Result<()> {
        log::trace!("running git {}", args.join(" "));
        self.command.run_checked(args)
    }
}

impl<'repo> Git for GitCommand<'repo> {
    fn run(&self, args: &[&str]) -> Result<()> {
        log::debug!("git {}", args.join(" "));
        self.command.run_checked(args)
    }

    fn tree_is_clean(&self) -> Result<bool> {
        self.command
            .run_stdout(&["status", "--short"])
            .map(|output| output.trim().is_empty())
    }

    fn has_branch(&self, branch: &str) -> Result<bool> {
        match self.find_branch(branch)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn on_branch(&self, branch: &str) -> Result<bool> {
        Ok(match self.repo.head()?.shorthand() {
            Some(name) => {
                dbg!(&name, &branch);
                name == branch
            }
            None => false,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::test;
    use serde::{Deserialize, Serialize};
    use std::panic;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TestData {
        v: String,
    }

    //     #[test]
    //     fn create_and_read_meta() {
    //         let (_dirs, git) = test::new_git();
    //         let data = "data".to_string();
    //         git.save(KEY, &data).expect("could not save data");
    //         let got: String = git.load(KEY).expect("could not load data");
    //         assert_eq!(got, data)
    //     }

    //     #[test]
    //     fn missing_meta_ok() {
    //         let (_od, _cd, git) = get_repo();
    //         let err = git.load::<String>(KEY).unwrap_err();
    //         match err {
    //             store::Error::Missing => {}
    //             _ => panic!("Expected missing"),
    //         }
    //     }

    //     #[test]
    //     fn corrupt_meta_is_error() {
    //         let (_od, _cd, git) = get_repo();

    //         let data = vec![1, 2, 3];
    //         git.save(KEY, &data).expect("could not save data");

    //         let err = git.load::<String>(KEY).unwrap_err();

    //         match err {
    //             store::Error::Format(_) => {}
    //             _ => panic!("Expected format error"),
    //         }
    //     }

    //     #[test]
    //     fn latest_meta_loaded() {
    //         let (_od, _cd, git) = get_repo();

    //         let data = TestData {
    //             v: "first".to_string(),
    //         };
    //         git.save(KEY, &data).expect("could not save data");

    //         let data = TestData {
    //             v: "second".to_string(),
    //         };
    //         git.save(KEY, &data).expect("could not save data");

    //         let got: TestData = git.load(KEY).expect("could not load data");
    //         assert_eq!(got.v, data.v)
    //     }

    //     #[test]
    //     fn push_conflict() {
    //         let (origin, _) = crate::test::repo_init();
    //         let (_clone1, clonerepo1) = crate::test::repo_clone(origin.path());
    //         let (_clone2, clonerepo2) = crate::test::repo_clone(origin.path());

    //         let git1 = GitCommand::from_repo(clonerepo1);
    //         let data = "first state".to_string();
    //         git1.save(KEY, &data).unwrap();

    //         let git2 = GitCommand::from_repo(clonerepo2);
    //         let data = "second state".to_string();
    //         let err = git2.save(KEY, &data).unwrap_err();
    //         match err {
    //             store::Error::Conflict(_) => {}
    //             _ => panic!("Pushing to remote twice should result in conflict"),
    //         }
    //     }
}
