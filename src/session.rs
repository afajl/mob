use crate::git;
use anyhow::{Error, Result};
use chrono::NaiveTime;
use chrono::{DateTime, Utc};
use dialoguer::Input;
use serde::{Deserialize, Serialize};
use std::default::Default;

const TIME_FORMAT: &str = "%H:%M";

type DurationMinutes = i64;

fn validate_clock(text: &str) -> Result<(), chrono::ParseError> {
    NaiveTime::parse_from_str(text, TIME_FORMAT).map(|_| ())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub commit_message: String,
    pub work_duration: DurationMinutes,
    pub break_duration: DurationMinutes,
    pub break_interval: DurationMinutes,
    pub lunch_start: String,
    pub lunch_end: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            commit_message: "mob synk [skip ci]".into(),
            work_duration: 10,
            break_interval: 55,
            break_duration: 5,
            lunch_start: "11:30".into(),
            lunch_end: "12:30".into(),
        }
    }
}

impl Settings {
    pub fn ask() -> Result<Self> {
        let default = Settings::default();

        let commit_message = Input::new()
            .with_prompt("Commit message")
            .default(default.commit_message)
            .interact()?;

        let work_duration = Input::new()
            .with_prompt("Work duration")
            .default(default.work_duration)
            .interact()?;

        let break_interval = Input::new()
            .with_prompt("Break interval")
            .default(default.break_interval)
            .interact()?;

        let break_duration = Input::new()
            .with_prompt("Break duration")
            .default(default.break_duration)
            .interact()?;

        let lunch_start = Input::new()
            .with_prompt("Lunch start")
            .default(default.lunch_start)
            .validate_with(validate_clock)
            .interact()?;

        let lunch_end = Input::new()
            .with_prompt("Lunch end")
            .default(default.lunch_end)
            .validate_with(validate_clock)
            .interact()?;

        let config = Self {
            commit_message,
            work_duration,
            break_interval,
            break_duration,
            lunch_start,
            lunch_end,
        };
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branches {
    pub branch: String,
    pub base_branch: String,
}

impl Branches {
    pub fn with_remote(&self, remote: &str) -> Self {
        Self {
            branch: format!("{}/{}", remote, self.branch),
            base_branch: format!("{}/{}", remote, self.base_branch),
        }
    }
    pub fn ask(default: Branches) -> Result<Branches> {
        let base_branch = Input::new()
            .with_prompt("Base branch")
            .default(default.base_branch)
            .interact()?;

        let branch = Input::new()
            .with_prompt("Feature branch")
            .default(default.branch)
            .interact()?;

        Ok(Branches {
            branch,
            base_branch,
        })
    }
}

impl Default for Branches {
    fn default() -> Self {
        Self {
            base_branch: "master".to_string(),
            branch: "mob-session".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drivers(Vec<String>);

impl Drivers {
    pub fn insert(mut self, name: &str) -> Self {
        if !self.0.contains(&name.to_string()) {
            self.0.push(name.to_string())
        }
        self
    }

    pub fn next(&self, current: &str) -> Option<String> {
        match self.0.len() {
            0 => panic!("Next driver called before anyone started"),
            1 => None,
            _ => Some(
                self.0
                    .iter()
                    .position(|name| name == current)
                    .map(|index| {
                        let next_index = (index + 1) % self.0.len();
                        self.0[next_index].clone()
                    })
                    .expect(
                        format!("Could not find current driver {} in drivers", current).as_str(),
                    ),
            ),
        }
    }

    pub fn remove(mut self, name: &str) -> Self {
        let index = self
            .0
            .iter()
            .position(|n| n == name)
            .expect(format!("Trying to remove {} that is not a driver", name).as_str());
        self.0.remove(index);
        self
    }
}

impl Default for Drivers {
    fn default() -> Self {
        Drivers(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum State {
    Stopped,
    Working { driver: String },
    Break { next: Option<String> },
    WaitingForNext { next: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub last_break: DateTime<Utc>,
    pub drivers: Drivers,
    pub branches: Branches,
    pub settings: Option<Settings>,
    pub state: State,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            last_break: Utc::now(),
            drivers: Drivers::default(),
            branches: Branches::default(),
            settings: None,
            state: State::Stopped,
        }
    }
}

pub trait Store {
    fn load(&self) -> Result<Session>;
    fn save(&self, session: &Session) -> Result<()>;
    fn clean(&self) -> Result<()>;
}

pub struct SessionStore<'a> {
    store: &'a dyn git::Store,
}

impl<'a> SessionStore<'a> {
    pub fn new(store: &'a impl git::Store) -> Self {
        SessionStore { store }
    }
}

impl<'a> Store for SessionStore<'a> {
    fn load(&self) -> Result<Session> {
        let res = self.store.load();
        match res {
            Ok(session) => Ok(session),
            Err(git::store::Error::Missing) => Ok(Session::default()),
            Err(git::store::Error::Format(cause)) => {
                log::error!("Could not parse meta data. You've probably updated mob");
                log::error!("Run 'mob clean' and run start again");
                Err(Error::from(cause))
            }
            Err(error) => Err(Error::from(error)),
        }
    }

    fn save(&self, session: &Session) -> Result<()> {
        self.store.save(session)?;
        Ok(())
    }

    fn clean(&self) -> Result<()> {
        self.store.clean()?;
        Ok(())
    }
}
