use crate::{config::Config, git, session, settings};
use anyhow::Result;
use clap::Clap;
use session::State;

#[derive(Clap, Debug)]
pub struct StartOpts {}

pub struct Start<'a> {
    config: Config,
    git: &'a dyn git::Git,
    settings: &'a dyn settings::Service,
    session: &'a mut dyn session::Service,
}

impl<'a> Start<'a> {
    pub fn new(
        config: Config,
        git: &'a impl git::Git,
        settings: &'a impl settings::Service,
        session: &'a mut impl session::Service,
    ) -> Start<'a> {
        Start {
            config,
            git,
            settings,
            session,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let me = self.config.name.clone();
        println!("me {}", me);
        match self.session.load()?.state {
            State::Stopped => {
                let current_branch = self.git.current_branch()?;
                let branches = session::Branches::ask(current_branch)?;
                let state = State::Working {
                    branches,
                    driver: me,
                };
                println!("Starting work, state: {:?}", state);
                self.start_new()?;
                self.session.set_state(state)
            }
            State::Working { driver, .. } if driver == me => {
                println!("You've already started working");
                Ok(())
            }
            State::Working { driver, .. } => {
                println!("{} is already driving!", driver);
                let take_over = dialoguer::Confirmation::new()
                    .with_text("Do you want to take over with the risk of losing work?")
                    .default(false)
                    .interact()?;
                if take_over {
                    return self.take_over();
                }
                Ok(())
            }
            State::Break {
                branches,
                until,
                next,
            } => unimplemented!(),
            State::WaitingForNext { branches, next } => unimplemented!(),
        }
    }

    fn start_new(&self) -> Result<()> {
        return Ok(());
    }

    fn take_over(&self) -> Result<()> {
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::{anyhow, Error, Result};
    use chrono::Utc;
    use session::{Branches, Session, State};

    struct MockSession {
        load_result: Result<session::Session>,
    }

    impl session::Service for MockSession {
        fn load(&self) -> Result<session::Session> {
            match &self.load_result {
                Ok(val) => Ok(val.clone()),
                Err(_) => Err(anyhow!("error")),
            }
        }
    }

    struct MockSettings {
        load_result: Result<settings::Settings>,
    }

    impl settings::Service for MockSettings {
        fn load(&self) -> Result<settings::Settings> {
            match &self.load_result {
                Ok(val) => Ok(val.clone()),
                Err(_) => Err(anyhow!("error")),
            }
        }
    }

    struct MockGit {}

    impl git::Git for MockGit {
        fn run(&self, args: &[&str]) -> Result<()> {
            unimplemented!()
        }
        fn tree_is_clean(&self) -> Result<bool> {
            unimplemented!()
        }
        fn has_branch(&self, branch: &str) -> Result<bool> {
            unimplemented!()
        }
        fn on_branch(&self, branch: &str) -> Result<bool> {
            unimplemented!()
        }
    }

    #[test]
    fn start_when_already_started() {
        let git = MockGit {};
        let session = Session::new();

        let branches = Branches {
            base_branch: "master".into(),
            branch: "mob".into(),
        };
        let session = Session {
            last_break: Utc::now(),
            state: State::Working {
                branches,
                driver: "me".into(),
            },
        };
        let settings = MockSettings {
            load_result: Err(anyhow!("Unused")),
        };
        let session = MockSession {
            load_result: Ok(session),
        };
        let start = Start::new(&MockGit {}, &settings, &session);

        start.run()
    }
}
