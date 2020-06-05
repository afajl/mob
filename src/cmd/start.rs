use crate::{config::Config, git, session, timer};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{self, Clap};
use session::State;

#[derive(Clap, Debug)]
pub struct StartOpts {
    /// How long you want this work session to last
    #[clap(name = "MINUTES")]
    minutes: Option<i64>,
}

pub struct Start<'a> {
    git: &'a dyn git::Git,
    store: &'a dyn session::Store,
    timer: &'a dyn timer::Timer,
    opts: StartOpts,
    config: Config,
}

impl<'a> Start<'a> {
    pub fn new(
        git: &'a impl git::Git,
        store: &'a impl session::Store,
        timer: &'a impl timer::Timer,
        opts: StartOpts,
        config: Config,
    ) -> Start<'a> {
        Self {
            git,
            store,
            timer,
            opts,
            config,
        }
    }

    pub fn run(&self) -> Result<()> {
        let me = &self.config.name;

        let mut session = self.store.load()?;

        match &session.state {
            State::Stopped => self.start_new(session)?,
            State::Working { driver } if driver == me.as_str() => {
                log::warn!("It's already your turn");
            }
            State::Working { driver } => {
                log::warn!("{} has not run mob next", driver);
                let selections = &["Retry", "Take turn with the risk of losing work"];
                let selection = dialoguer::Select::new()
                    .with_prompt("What do you want to do?")
                    .default(0)
                    .items(&selections[..])
                    .interact()?;

                match selection {
                    0 => return self.run(),
                    _ => self.start(session)?,
                }
            }
            State::WaitingForNext {
                next: Some(driver),
                is_break,
            } if driver == me.as_str() => {
                if *is_break {
                    session.last_break = Utc::now();
                }
                self.start(session)?
            }
            State::WaitingForNext {
                next: None,
                is_break,
            } => {
                if *is_break {
                    session.last_break = Utc::now();
                }
                self.start(session)?
            }
            State::WaitingForNext {
                next: Some(driver),
                is_break,
            } => {
                if *is_break {
                    session.last_break = Utc::now();
                }
                if !session.drivers.contains(self.config.name.as_str()) {
                    self.start(session)?
                } else {
                    self.take_over(driver, session.clone())?
                }
            }
        };

        Ok(())
    }

    fn take_over(&self, from: &str, session: session::Session) -> Result<()> {
        let take_and_remove = format!("Take turn and remove {} from the mob", from);
        let selections = &["Take turn", take_and_remove.as_str(), "Abort"];
        let selection = dialoguer::Select::new()
            .with_prompt(format!("It's {}s turn. What do you want to do?", from).as_str())
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selection {
            0 => self.start(session),
            1 => {
                let session = session::Session {
                    drivers: session.drivers.remove(from),
                    ..session
                };
                self.start(session)
            }
            _ => Ok(()),
        }
    }

    fn start(&self, session: session::Session) -> Result<()> {
        if !self.git.tree_is_clean()? {
            return Err(anyhow!("Working tree is not clean"));
        }

        self.git
            .run(&["checkout", session.branches.base_branch.as_str()])?;
        self.git.run(&["fetch", "--all", "--prune"])?;

        if self.git.has_branch(session.branches.branch.as_str())? {
            self.git
                .run(&["branch", "-D", session.branches.branch.as_str()])?;
        }

        self.git
            .run(&["checkout", session.branches.branch.as_str()])?;

        let previous_driver = session.get_driver();
        let session = session::Session {
            state: State::Working {
                driver: self.config.name.clone(),
            },
            last_break: self.maybe_reset_break(session.last_break),
            drivers: session
                .drivers
                .insert(previous_driver, self.config.name.as_str()),
            ..session
        };

        self.store.save(&session)?;

        self.start_timer(
            session.settings.unwrap().work_duration,
            session.drivers.next(&self.config.name.as_str()),
        )
    }

    fn start_new(&self, session: session::Session) -> Result<()> {
        if !self.git.tree_is_clean()? {
            return Err(anyhow!("Working tree is not clean"));
        }

        let previous_driver = session.get_driver();

        let settings = match session.settings {
            Some(settings) => settings,
            None => session::Settings::ask()?,
        };

        let branches = session::Branches::ask(session.branches)?;

        let remote_branches = branches.with_remote(&self.config.remote);

        self.git.run(&["fetch", "--all", "--prune"])?;

        self.git.run(&["checkout", branches.base_branch.as_str()])?;
        self.git
            .run(&["merge", remote_branches.base_branch.as_str(), "--ff-only"])?;

        self.setup_branch(&branches, &remote_branches)?;

        let session = session::Session {
            state: State::Working {
                driver: self.config.name.clone(),
            },
            last_break: self.maybe_reset_break(session.last_break),
            drivers: session
                .drivers
                .insert(previous_driver, &self.config.name.as_str()),
            settings: Some(settings),
            branches,
        };

        self.store.save(&session)?;

        self.start_timer(
            session.settings.unwrap().work_duration,
            session.drivers.next(&self.config.name.as_str()),
        )
    }

    fn setup_branch(
        &self,
        branches: &session::Branches,
        remote_branches: &session::Branches,
    ) -> Result<()> {
        let create_and_push = || -> Result<()> {
            self.git
                .run(&["checkout", "-b", branches.branch.as_str()])?;

            self.git.run(&[
                "push",
                "--no-verify",
                "--set-upstream",
                self.config.remote.as_str(),
                branches.branch.as_str(),
            ])?;
            Ok(())
        };

        let has_local_branch = self.git.has_branch(branches.branch.as_str())?;
        let has_remote_branch = self.git.has_branch(remote_branches.branch.as_str())?;

        match (has_local_branch, has_remote_branch) {
            (true, true) => {
                let prompt = format!("Remote and local branch {} already exists", branches.branch);
                let selections = &[
                    "Use these branches",
                    "Remove local branch and checkout remote",
                    "Delete local and remote branch and start fresh",
                ];
                let selection = dialoguer::Select::new()
                    .with_prompt(prompt)
                    .default(0)
                    .items(&selections[..])
                    .interact()?;

                match selection {
                    0 => {
                        self.git.run(&["checkout", branches.branch.as_str()])?;
                    }

                    1 => {
                        self.git.run(&["branch", "-D", branches.branch.as_str()])?;
                        self.git.run(&["checkout", branches.branch.as_str()])?;
                    }
                    _ => {
                        self.git.run(&["branch", "-D", branches.branch.as_str()])?;
                        self.git.run(&[
                            "push",
                            &self.config.remote,
                            "--delete",
                            branches.branch.as_str(),
                            "--no-verify",
                        ])?;

                        create_and_push()?;
                    }
                }
            }
            (true, false) => {
                let prompt = format!(
                    "Local branch {} already exists but not remote",
                    branches.branch
                );
                let selections = &["Push local branch", "Delete local branch and start fresh"];
                let selection = dialoguer::Select::new()
                    .with_prompt(prompt)
                    .default(0)
                    .items(&selections[..])
                    .interact()?;

                match selection {
                    0 => {
                        self.git.run(&[
                            "push",
                            "--no-verify",
                            "--set-upstream",
                            self.config.remote.as_str(),
                            branches.branch.as_str(),
                        ])?;
                        self.git.run(&["checkout", branches.branch.as_str()])?;
                    }
                    _ => {
                        self.git.run(&["branch", "-D", branches.branch.as_str()])?;

                        create_and_push()?;
                    }
                }
            }
            (false, true) => {
                let prompt = format!("Remote branch {} already exists", branches.branch);
                let selections = &[
                    "Checkout remote branch",
                    "Delete remote branch and start fresh",
                ];
                let selection = dialoguer::Select::new()
                    .with_prompt(prompt)
                    .default(0)
                    .items(&selections[..])
                    .interact()?;

                match selection {
                    0 => {
                        self.git.run(&["checkout", branches.branch.as_str()])?;
                    }
                    _ => {
                        self.git.run(&[
                            "push",
                            &self.config.remote,
                            "--delete",
                            branches.branch.as_str(),
                            "--no-verify",
                        ])?;

                        create_and_push()?;
                    }
                }
            }
            (false, false) => {
                create_and_push()?;
            }
        };
        Ok(())
    }

    fn maybe_reset_break(&self, last_break: DateTime<Utc>) -> DateTime<Utc> {
        if Utc::now() - last_break > Duration::hours(8) {
            log::trace!("resetting last brake after 8 hours");
            return Utc::now();
        }
        last_break
    }

    fn start_timer(&self, minutes: i64, next: Option<String>) -> Result<()> {
        let minutes = self.opts.minutes.unwrap_or(minutes);

        let timer_message = format!(
            "mob next {}",
            match next {
                Some(name) => name,
                None => "".to_string(),
            }
        );

        self.timer.start(
            "Your turn",
            chrono::Duration::minutes(minutes),
            timer_message.as_str(),
        )?;
        log::info!("Done. Run mob next");
        Ok(())
    }
}
