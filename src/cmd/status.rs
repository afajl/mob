use crate::{config::Config, git, session};
use anyhow::Result;
use clap::{self, Clap};
use session::State;

#[derive(Clap, Debug)]
pub struct StatusOpts {
    /// Show raw status
    #[clap(short, long)]
    raw: bool,
}

pub struct Status<'a> {
    store: &'a dyn session::Store,
    config: Config,
    opts: StatusOpts,
}

impl<'a> Status<'a> {
    pub fn new(opts: StatusOpts, store: &'a impl session::Store, config: Config) -> Status<'a> {
        Self {
            opts,
            store,
            config,
        }
    }

    pub fn run(&self) -> Result<()> {
        let session = self.store.load()?;

        if self.opts.raw {
            println!("{:#?}", session);
            return Ok(());
        }

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
}
