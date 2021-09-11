use crate::{config::Config, git, session};
use anyhow::Result;
use session::State;

pub struct Done<'a> {
    git: &'a dyn git::Git,
    store: &'a dyn session::Store,
    config: Config,
}

impl<'a> Done<'a> {
    pub fn new(git: &'a impl git::Git, store: &'a impl session::Store, config: Config) -> Done<'a> {
        Self { git, store, config }
    }

    pub fn run(&self) -> Result<()> {
        let me = self.config.name.clone();

        let session = self.store.load()?;
        match &session.state {
            State::Stopped => {
                log::warn!("No current mob session, run mob start");
            }
            State::Working { driver } if driver == me.as_str() => self.done(session)?,
            State::Working { driver } => {
                log::warn!("{} is currently working", driver);
                let take_over = dialoguer::Confirm::new()
                    .with_prompt("Merge anyway with risk of loosing work?")
                    .default(false)
                    .interact()?;

                if take_over {
                    self.done(session)?;
                }
            }
            State::WaitingForNext { .. } => self.done(session)?,
        }
        Ok(())
    }

    fn done(&self, session: session::Session) -> Result<()> {
        if !self.git.tree_is_clean()? {
            log::info!("Working tree is dirty, committing first");

            if !self.on_branch(session.branches.branch.as_str())? {
                log::error!(
                    "You must be on branch {} to run done",
                    session.branches.branch.as_str()
                );
                return Ok(());
            }

            self.git.run(&["add", "--all"])?;
            self.git.run(&[
                "commit",
                "--message",
                session.settings.as_ref().unwrap().commit_message.as_str(),
                "--no-verify",
            ])?;
        }

        let remote_branches = session.branches.with_remote(&self.config.remote);

        self.git.run(&[
            "push",
            "--no-verify",
            self.config.remote.as_str(),
            session.branches.branch.as_str(),
        ])?;

        self.git.run(&["fetch", "--all", "--prune"])?;
        self.git
            .run(&["checkout", session.branches.base_branch.as_str()])?;

        self.git
            .run(&["merge", remote_branches.base_branch.as_str(), "--ff-only"])?;

        self.git.run(&[
            "merge",
            "--squash",
            "--ff",
            session.branches.branch.as_str(),
        ])?;

        // Delete mob branch
        self.git
            .run(&["branch", "-D", session.branches.branch.as_str()])?;
        self.git.run(&[
            "push",
            "--no-verify",
            &self.config.remote,
            "--delete",
            session.branches.branch.as_str(),
        ])?;

        log::info!("Run git diff --staged and then");
        log::info!("git commit -m 'describe what changed'");

        let session = session::Session {
            state: State::Stopped,
            ..session
        };
        self.store.save(session)?;
        Ok(())
    }

    fn on_branch(&self, branch: &str) -> Result<bool> {
        Ok(match self.git.current_branch()? {
            Some(name) => name == branch,
            None => false,
        })
    }
}
