pub mod store;
use crate::command;
use crate::os;
use anyhow::{Result, anyhow};
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

pub struct GitCommand {
    command: command::Command<'static>,
    pub remote: String,
}

impl GitCommand {
    pub fn new(path: Option<PathBuf>, remote: String) -> Result<GitCommand> {
        let path = path.unwrap_or(env::current_dir()?);

        // Verify we're in a git repository by checking for .git directory
        let command = command::Command::new(os::command("git")).working_directory(path.as_path());
        command
            .run_stdout(["rev-parse", "--git-dir"])
            .map_err(|_| anyhow!("Not a git repository: {}", path.display()))?;

        Ok(Self { command, remote })
    }

    fn last_commit_oid(&self, reference: &str) -> Option<String> {
        let absolute_ref = format!("refs/heads/{}", reference);

        self.command
            .run_stdout(["rev-parse", "--verify", "--quiet", &absolute_ref])
            .ok()
            .map(|s| s.trim().to_string())
    }

    pub fn create_commit(&self, commit: &CommitFile) -> Result<String> {
        // 1. Create blob from data using git hash-object
        let blob_oid = self
            .command
            .run_with_stdin(["hash-object", "-w", "--stdin"], commit.data)?;
        let blob_oid = blob_oid.trim();

        // 2. Create tree using git mktree
        // Format: <mode> <type> <hash>\t<filename>
        let tree_entry = format!("100644 blob {}\t{}\n", blob_oid, commit.filename);
        let tree_oid = self
            .command
            .run_with_stdin(["mktree"], tree_entry.as_bytes())?;
        let tree_oid = tree_oid.trim();

        // 3. Create commit using git commit-tree
        let parent = self.last_commit_oid(commit.reference);
        let commit_oid = match parent {
            Some(parent_oid) => self.command.run_with_stdin(
                [
                    "commit-tree",
                    tree_oid,
                    "-p",
                    &parent_oid,
                    "-m",
                    commit.message,
                ],
                &[],
            )?,
            None => self
                .command
                .run_with_stdin(["commit-tree", tree_oid, "-m", commit.message], &[])?,
        };
        let commit_oid = commit_oid.trim().to_string();

        // 4. Update the reference to point to the new commit
        let absolute_ref = format!("refs/heads/{}", commit.reference);
        self.command
            .run_checked(["update-ref", &absolute_ref, &commit_oid])?;

        Ok(commit_oid)
    }

    pub fn show_file(&self, reference: &str, filename: &str) -> Result<Vec<u8>> {
        let spec = format!("{}:{}", reference, filename);
        let output = self.command.run(["show", &spec])?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to read {}:{}: {}",
                reference,
                filename,
                output.stderr
            ));
        }

        Ok(output.stdout.into_bytes())
    }

    fn run_quietly(&self, args: &[&str]) -> Result<()> {
        log::trace!("running git {}", args.join(" "));
        self.command.run_checked(args)
    }
}

impl Git for GitCommand {
    fn run(&self, args: &[&str]) -> Result<()> {
        self.command.run_checked(args)
    }

    fn tree_is_clean(&self) -> Result<bool> {
        self.dirty_files().map(|output| output.trim().is_empty())
    }

    fn has_branch(&self, branch: &str) -> Result<bool> {
        let output = self
            .command
            .run(["rev-parse", "--verify", "--quiet", branch])?;
        Ok(output.status.success())
    }

    fn current_branch(&self) -> Result<Option<String>> {
        let output = self.command.run(["rev-parse", "--abbrev-ref", "HEAD"])?;

        if !output.status.success() {
            return Ok(None);
        }

        let branch = output.stdout.trim();
        if branch == "HEAD" {
            // Detached HEAD state
            Ok(None)
        } else {
            Ok(Some(branch.to_string()))
        }
    }

    fn dirty_files(&self) -> Result<String> {
        self.command.run_stdout(["status", "--short"])
    }
}
