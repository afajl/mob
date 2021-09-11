use super::latest::*;
use super::v0::{SessionV0, StateV0};
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub drivers: Drivers,
    pub branches: Branches,
    pub settings: Option<Settings>,
    pub state: State,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            drivers: Drivers::default(),
            branches: Branches::default(),
            settings: None,
            state: State::Stopped,
        }
    }
}

impl From<SessionV0> for Session {
    fn from(session_v0: SessionV0) -> Self {
        Self {
            drivers: session_v0.drivers,
            branches: session_v0.branches,
            settings: match session_v0.settings {
                Some(settings) => Some(Settings {
                    commit_message: settings.commit_message,
                    work_duration: settings.work_duration,
                }),
                None => None,
            },
            state: match session_v0.state {
                StateV0::Stopped => State::Stopped,
                StateV0::Working { driver } => State::Working { driver },
                StateV0::WaitingForNext { next, .. } => State::WaitingForNext { next },
            },
        }
    }
}

impl Session {
    pub fn get_driver(&self) -> Option<String> {
        match &self.state {
            State::Working { driver } => Some(driver.clone()),
            State::WaitingForNext { next, .. } => next.clone(),
            _ => None,
        }
    }
}
