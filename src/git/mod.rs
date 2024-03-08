pub mod store;
use crate::command;
use crate::os;
use anyhow::Result;
use std::env;
use std::path::PathBuf;
pub use store::Store;

pub trait Git {
    fn run(&self, args: &[&str]) -> Result<()>;
    fn tree_is_clean(&self) -> Result<bool>;
    fn has_branch(&self, branch: &str) -> Result<bool>;
    fn current_branch(&self) -> Result<Option<String>>;
    fn dirty_files(&self) -> Result<String>;
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
    pub remote: String,
}

impl<'repo> GitCommand<'repo> {
    pub fn new(path: Option<PathBuf>, remote: String) -> Result<GitCommand<'repo>> {
        let path = path.unwrap_or(env::current_dir()?);
        let command = command::Command::new(os::command("git")).working_directory(path.as_path());
        Ok(Self { command, remote })
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

    pub fn create_commit(&self, commit: &CommitFile) -> Result<(), anyhow::Error> {
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
        self.dirty_files().map(|output| output.trim().is_empty())
    }

    fn has_branch(&self, branch: &str) -> Result<bool> {
        match self.find_branch(branch)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn current_branch(&self) -> Result<Option<String>> {
        return Ok(self.repo.head()?.shorthand().map(String::from));
    }

    fn dirty_files(&self) -> Result<String> {
        self.command.run_stdout(["status", "--short"])
    }
}
