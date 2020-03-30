use anyhow::{Context, Error, Result};
use chrono::NaiveTime;
use dialoguer::Input;
use serde::{Deserialize, Serialize};
use std::fs::File;

const MOB_FILE: &str = ".mob";
const TIME_FORMAT: &str = "%H:%M";

type DurationMinutes = i64;

fn validate_clock(text: &str) -> Result<(), chrono::ParseError> {
    NaiveTime::parse_from_str(text, TIME_FORMAT).map(|_| ())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub break_duration: DurationMinutes,
    pub lunch_start: String,
    pub mob_branch: String,
    pub base_branch: String,
    pub remote: String,
}

impl Config {
    pub fn load() -> Result<Config> {
        File::open(MOB_FILE)
            .map_err(Error::from)
            .and_then(|f| serde_yaml::from_reader(f).map_err(Error::from))
            .context(format!("failed to load {}", MOB_FILE))
    }

    fn save(&self) -> Result<()> {
        File::create(MOB_FILE)
            .map_err(Error::from)
            .and_then(|f| serde_yaml::to_writer(f, &self).map_err(Error::from))
            .with_context(|| format!("failed to save {}", MOB_FILE))
    }

    pub fn remote_mob_branch(&self) -> String {
        [self.remote.clone(), self.mob_branch.clone()].join("/")
    }

    pub fn remote_base_branch(&self) -> String {
        [self.remote.clone(), self.base_branch.clone()].join("/")
    }

    pub fn init_config() -> Result<Config> {
        let break_duration = Input::new()
            .with_prompt("Break duration")
            .default(5)
            .interact()?;

        let lunch_start = Input::new()
            .with_prompt("Lunch start")
            .default("11:30".to_string())
            .validate_with(validate_clock)
            .interact()?;

        let mob_branch = Input::new()
            .with_prompt("Mob branch")
            .default("mob-session".to_string())
            .interact()?;

        // TODO: default to current branch
        let base_branch = Input::new()
            .with_prompt("Base branch")
            .default("master".to_string())
            .interact()?;

        let remote = Input::new()
            .with_prompt("Remote")
            .default("origin".to_string())
            .interact()?;

        let config = Config {
            break_duration,
            lunch_start,
            mob_branch,
            base_branch,
            remote,
        };
        config.save()?;
        return Ok(config);
    }
}
