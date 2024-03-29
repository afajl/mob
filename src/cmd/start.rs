use crate::{command, config::Config, git, session, timer};
use anyhow::{anyhow, Result};
use clap::Parser;
use session::State;

#[derive(Parser, Debug)]
pub struct StartOpts {
    /// How long you want this work session to last
    #[clap(name = "MINUTES")]
    minutes: Option<i64>,
}

pub struct Start<'a> {
    git: &'a dyn git::Git,
    store: &'a dyn session::Store,
    opts: StartOpts,
    config: Config,
}

impl<'a> Start<'a> {
    pub fn new(
        git: &'a impl git::Git,
        store: &'a impl session::Store,
        opts: StartOpts,
        config: Config,
    ) -> Start<'a> {
        Self {
            git,
            store,
            opts,
            config,
        }
    }

    pub fn run(&self) -> Result<()> {
        let me = &self.config.name;
        command::run_hook(&self.config.hooks.before_start, me, "")?;

        self.is_clean()?;

        let session = self.store.load_or_default()?;

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
            State::WaitingForNext { next: Some(driver) } if driver == me.as_str() => {
                self.start(session)?;
            }
            State::WaitingForNext { next: None } => self.start(session)?,
            State::WaitingForNext { next: Some(driver) } => {
                if session.drivers.contains(self.config.name.as_str()) {
                    self.take_over(driver, session.clone())?;
                } else {
                    self.start(session)?;
                }
            }
        };

        Ok(())
    }

    fn is_clean(&self) -> Result<()> {
        let status = self.git.dirty_files()?;
        if status.is_empty() {
            return Ok(());
        }

        log::warn!("Working tree is dirty:\n{}", status);

        let selection = dialoguer::Select::new()
            .default(0)
            .items(&["Quit", "Stash changes", "Discard changes"])
            .interact()?;

        match selection {
            0 => Err(anyhow!("Working tree is not clean")),
            1 => self.git.run(&["stash"]),
            2 => self.git.run(&["reset", "HEAD", "--hard"]),
            _ => unreachable!("could not come here"),
        }
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
                    state: State::WaitingForNext {
                        next: Some(self.config.name.clone()),
                    },
                    ..session
                };
                self.start(session)
            }
            _ => Ok(()),
        }
    }

    fn start(&self, session: session::Session) -> Result<()> {
        self.git.run(&["fetch", "--all", "--prune"])?;

        let remote_branches = session.branches.with_remote(&self.config.remote);
        self.git.run(&[
            "switch",
            "--force-create",
            session.branches.branch.as_str(),
            remote_branches.branch.as_str(),
        ])?;

        let previous_driver = session.get_driver();

        let session = session::Session {
            state: State::Working {
                driver: self.config.name.clone(),
            },
            drivers: session
                .drivers
                .insert(previous_driver, self.config.name.as_str()),
            ..session
        };

        let next_driver = session.drivers.next(self.config.name.as_str());
        let work_duration = session.settings.as_ref().unwrap().work_duration;

        self.store.save(session)?;

        let next_driver_name = next_driver.unwrap_or_else(|| String::from("anyone"));
        self.start_timer(work_duration, &next_driver_name)
    }

    fn start_new(&self, session: session::Session) -> Result<()> {
        let previous_driver = session.get_driver();

        let settings = match session.settings {
            Some(settings) => settings,
            None => session::Settings::ask()?,
        };

        let default_branches = session::Branches {
            base_branch: self
                .git
                .current_branch()
                .unwrap_or(None)
                .unwrap_or(session.branches.base_branch),
            ..session.branches
        };

        let branches = session::Branches::ask(default_branches)?;

        let remote_branches = branches.with_remote(&self.config.remote);

        self.git.run(&["fetch", "--all", "--prune"])?;

        if !self.git.has_branch(remote_branches.base_branch.as_str())? {
            return Err(anyhow!(
                "You need to push your branch `{}` first",
                branches.base_branch
            ));
        }

        self.git.run(&["checkout", branches.base_branch.as_str()])?;
        self.git
            .run(&["merge", remote_branches.base_branch.as_str(), "--ff-only"])?;

        self.setup_branch(&branches, &remote_branches)?;

        let session = session::Session {
            state: State::Working {
                driver: self.config.name.clone(),
            },
            drivers: session
                .drivers
                .insert(previous_driver, self.config.name.as_str()),
            settings: Some(settings),
            branches,
        };

        self.store.save(session.clone())?;

        self.start_timer(session.settings.unwrap().work_duration, "anyone")
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

                if selection == 0 {
                    self.git.run(&[
                        "push",
                        "--no-verify",
                        "--set-upstream",
                        self.config.remote.as_str(),
                        branches.branch.as_str(),
                    ])?;
                    self.git.run(&["checkout", branches.branch.as_str()])?;
                } else {
                    self.git.run(&["branch", "-D", branches.branch.as_str()])?;

                    create_and_push()?;
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

    fn start_timer(&self, minutes: i64, next_driver: &str) -> Result<()> {
        let minutes = self.opts.minutes.unwrap_or(minutes);

        let current_driver = self.config.name.as_str();
        command::run_hook(&self.config.hooks.after_start, current_driver, next_driver)?;
        timer::start("Your turn", chrono::Duration::minutes(minutes))?;
        log::info!("Done. Run mob next");
        command::run_hook(&self.config.hooks.after_timer, current_driver, next_driver)
    }
}
