use crate::{config::Config, git, session};
use anyhow::Result;
use session::State;

pub struct Next<'a> {
    git: &'a dyn git::Git,
    store: &'a dyn session::Store,
    config: Config,
}

impl<'a> Next<'a> {
    pub fn new(git: &'a impl git::Git, store: &'a impl session::Store, config: Config) -> Next<'a> {
        Self { git, store, config }
    }

    pub fn run(&self) -> Result<()> {
        let me = &self.config.name;

        let session = self.store.load()?;
        match &session.state {
            State::Stopped => {
                log::warn!("No current mob session, run mob start");
            }
            State::Working { driver } if driver != me.as_str() => {
                log::warn!("The current driver is {}", driver);
            }
            State::Working { .. } => self.next(session)?,
            State::WaitingForNext { next, .. } => {
                match next {
                    Some(name) if name == me.as_str() => log::info!("It's your turn. Run start"),
                    Some(name) => log::info!("Waiting for {} to start", name),
                    None => log::info!("Waiting for someone to run start"),
                };
            }
        };
        Ok(())
    }

    fn next(&self, session: session::Session) -> Result<()> {
        if self.git.tree_is_clean()? {
            log::info!("Nothing was changed, so nothing to commit");
        } else {
            self.git.run(&["add", "--all"])?;
            self.git.run(&[
                "commit",
                "--message",
                session.settings.as_ref().unwrap().commit_message.as_str(),
                "--no-verify",
            ])?;

            self.git.run(&[
                "push",
                "--no-verify",
                self.config.remote.as_str(),
                session.branches.branch.as_str(),
            ])?;
        }

        let next_driver = session.drivers.next(&self.config.name);
        let next_driver_name = match next_driver {
            Some(ref driver) => driver,
            None => "anyone!",
        };

        log::info!("Next driver: {}", next_driver_name);
        Ok(())
    }
}
