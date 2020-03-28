use crate::command;
use crate::os;
use anyhow::Result;
use log;
use std::path::PathBuf;

pub struct Git {
    command: command::Command<'static>,
    dry_run: bool,
}

impl Git {
    pub fn new(dry_run: bool) -> Self {
        Self {
            command: command::Command::new(os::command("git")),
            dry_run,
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
