use crate::{config::Config, git, session, timer};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{self, Clap};
use session::State;

#[derive(Clap, Debug)]
pub struct StartOpts {
    #[clap(name = "base-branch")]
    base_branch: Option<String>,

    #[clap(name = "branch")]
    branch: Option<String>,

    #[clap(short = "q")]
    quiet: bool,
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
                log::warn!("{} is already driving!", driver);
                let take_over = dialoguer::Confirmation::new()
                    .with_text("Do you want to take over with the risk of losing work?")
                    .default(false)
                    .interact()?;
                if take_over {
                    self.start(session)?
                }
            }
            State::Break { next: Some(driver) } if driver == me.as_str() => {
                session.last_break = Utc::now();
                self.start(session)?
            }
            State::Break { next: None } => {
                session.last_break = Utc::now();
                self.start(session)?
            }
            State::Break { next: Some(driver) } => self.take_over(driver, session.clone())?,
            State::WaitingForNext { next: Some(driver) } if driver == me.as_str() => {
                self.start(session)?
            }
            State::WaitingForNext { next: None } => self.start(session)?,
            State::WaitingForNext { next: Some(driver) } => {
                self.take_over(driver, session.clone())?
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
        self.git
            .run(&["branch", "-D", session.branches.branch.as_str()])?;
        self.git
            .run(&["checkout", session.branches.branch.as_str()])?;

        let session = session::Session {
            state: State::Working {
                driver: self.config.name.clone(),
            },
            last_break: self.maybe_reset_break(session.last_break),
            drivers: session.drivers.insert(self.config.name.as_str()),
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

        let settings = match session.settings {
            Some(settings) => settings,
            None => session::Settings::ask()?,
        };

        let branches = match (&self.opts.base_branch, &self.opts.branch) {
            (Some(base_branch), Some(branch)) => session::Branches {
                branch: branch.clone(),
                base_branch: base_branch.clone(),
            },
            _ => {
                if self.opts.quiet {
                    return Err(anyhow!("Missing branches"));
                }
                session::Branches::ask(session.branches)?
            }
        };

        let remote_branches = branches.with_remote(&self.config.remote);

        self.git.run(&["fetch", "--all", "--prune"])?;

        self.git.run(&["checkout", branches.base_branch.as_str()])?;
        self.git
            .run(&["merge", remote_branches.base_branch.as_str(), "--ff-only"])?;

        if self.git.has_branch(branches.branch.as_str())? {
            let message = format!("Branch {} already exists", branches.branch);
            if self.opts.quiet {
                return Err(anyhow!(message));
            }

            let clear_branch = dialoguer::Confirmation::new()
                .with_text((message + ". Remove branch?").as_str())
                .default(false)
                .interact()?;
            if clear_branch {
                self.git.run(&["branch", "-D", branches.branch.as_str()])?;
                self.git.run(&[
                    "push",
                    &self.config.remote,
                    "--delete",
                    branches.branch.as_str(),
                ])?;
            }
        }

        self.git
            .run(&["checkout", "-b", branches.branch.as_str()])?;

        self.git.run(&[
            "push",
            "--set-upstream",
            self.config.remote.as_str(),
            branches.branch.as_str(),
        ])?;

        let session = session::Session {
            state: State::Working {
                driver: self.config.name.clone(),
            },
            last_break: self.maybe_reset_break(session.last_break),
            drivers: session.drivers.insert(&self.config.name.as_str()),
            settings: Some(settings),
            branches,
        };

        self.store.save(&session)?;

        self.start_timer(
            session.settings.unwrap().work_duration,
            session.drivers.next(&self.config.name.as_str()),
        )
    }

    fn maybe_reset_break(&self, last_break: DateTime<Utc>) -> DateTime<Utc> {
        if Utc::now() - last_break > Duration::hours(8) {
            return Utc::now();
        }
        last_break
    }

    fn start_timer(&self, duration: i64, next: Option<String>) -> Result<()> {
        let timer_message = format!(
            "mob next {}",
            match next {
                Some(name) => name,
                None => "".to_string(),
            }
        );

        self.timer.start(
            "Your turn",
            chrono::Duration::minutes(duration),
            timer_message.as_str(),
        )?;
        log::info!("Done. Run mob next");
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{config, test, test::MockGit};
//     use anyhow::Result;
//     use git::CommitFile;
//     use session::Session;

//     #[test
//     ]
//j
//
//     fn start_new_ends_up_on_right_branch() -> Result<()> {
//         let opts = StartOpts {
//             base_branch: Some("master".to_string()),
//             branch: Some("test-branch".to_string()),
//             quiet: true,
//         };

//         let config = config::Config::default();
//         let git = MockGit::new();

//         let start = Start::new(&git, opts, config);

//         let session = Session::default();
//         let new_session = start.run(session)?;

//         assert_eq!(new_session.branches.base_branch, "master".to_string());
//         assert_eq!(new_session.branches.branch, "test-branch".to_string());

//         Ok(())
//     }

//     #[test]
//     fn start_new_fails_when_branch_exists() -> Result<()> {
//         let base_branch = "master".to_string();
//         let opts = StartOpts {
//             base_branch: Some(base_branch.clone()),
//             branch: Some("test-branch".to_string()),
//             quiet: true,
//         };

//         let config = config::Config::default();

//         let (_dirs, git) = test::new_git();

//         let commit = CommitFile {
//             filename: "file",
//             data: "knas".as_bytes(),
//             message: "msg",
//             reference: "refs/heads/test-branch",
//         };

//         git.create_commit(commit)?;

//         let start = Start::new(&git, opts, config);

//         let session = Session::default();
//         match start.run(session) {
//             Ok(_) => panic!("Should not create new branch in quiet mode"),
//             Err(_) => Ok(()),
//         }
//     }

//     // #[test]
//     // fn start_new_missing_branch() -> Result<()> {
//     //     let opts = StartOpts {
//     //         base_branch: Some("test-base".to_string()),
//     //         branch: None,
//     //         quiet: true,
//     //     };

//     //     let config = config::Config::default();
//     //     let settings = Settings::default();

//     //     let git = MockGit::new();
//     //     let start = Start::new(&git, opts, config, settings);

//     //     let session = Session::default();
//     //     match start.run(session) {
//     //         Ok(_) => panic!("Missing branch should fail"),
//     //         Err(_) => Ok(()),
//     //     }
//     // }
// }
