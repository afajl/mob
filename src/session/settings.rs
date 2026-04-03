use crate::prompt::Prompter;
use anyhow::Result;
use serde::{Deserialize, Serialize};

type DurationMinutes = i64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub commit_message: String,
    pub work_duration: DurationMinutes,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            commit_message: "mob sync [skip ci]".into(),
            work_duration: 10,
        }
    }
}

impl Settings {
    pub fn ask(prompter: &dyn Prompter) -> Result<Self> {
        let default = Settings::default();

        let commit_message = prompter.input_string("Commit message", &default.commit_message)?;

        let work_duration = prompter.input_i64("Work duration", default.work_duration)?;

        let config = Self {
            commit_message,
            work_duration,
        };
        Ok(config)
    }
}
