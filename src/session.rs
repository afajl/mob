use crate::store;
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use dialoguer::Input;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branches {
    pub branch: String,
    pub base_branch: String,
}

impl Branches {
    pub fn ask(current_branch: String) -> Result<Branches> {
        let base_branch = Input::new()
            .with_prompt("Base branch")
            .default(current_branch)
            .interact()?;

        let branch = Input::new()
            .with_prompt("Feature branch")
            .default("mob-session".into())
            .interact()?;

        Ok(Branches {
            branch,
            base_branch,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum State {
    Stopped,
    Working {
        branches: Branches,
        driver: String,
    },
    Break {
        branches: Branches,
        until: i64,
        next: String,
    },
    WaitingForNext {
        branches: Branches,
        next: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub last_break: DateTime<Utc>,
    pub state: State,
}

impl Session {
    pub fn new() -> Session {
        Session {
            last_break: Utc::now(),
            state: State::Stopped,
        }
    }
}

pub trait Service {
    fn load(&self) -> Result<Session>;
    fn set_state(&mut self, state: State) -> Result<()>;
}

const STORE_KEY: store::Key = store::Key::Session;

pub struct StoreService<'a> {
    session: Option<Session>,
    store: &'a dyn store::Store<Session>,
}

impl<'a> StoreService<'a> {
    pub fn new(store: &'a impl store::Store<Session>) -> Self {
        StoreService {
            store,
            session: None,
        }
    }
}

impl<'a> Service for StoreService<'a> {
    fn load(&self) -> Result<Session> {
        if let Some(session) = &self.session {
            return Ok(session.clone());
        }
        let res = self.store.load(STORE_KEY);
        match res {
            Ok(session) => Ok(session),
            Err(store::Error::Missing) => Ok(Session::new()),
            Err(error) => Err(Error::from(error)),
        }
    }

    fn set_state(&mut self, state: State) -> Result<()> {
        let session = self.load()?;
        let new_session = Session { state, ..session };

        self.store.save(STORE_KEY, &new_session)?;
        self.session = Some(session);
        Ok(())
    }
}
