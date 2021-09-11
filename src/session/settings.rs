use anyhow::Result;
use dialoguer::Input;
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

        let config = Self {
            commit_message,
            work_duration,
        };
        Ok(config)
    }
}
