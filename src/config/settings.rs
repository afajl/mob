use crate::store;
use anyhow::{anyhow, Error, Result};
use chrono::NaiveTime;
use dialoguer::Input;
use serde::{Deserialize, Serialize};

const TIME_FORMAT: &str = "%H:%M";

type DurationMinutes = i64;

fn validate_clock(text: &str) -> Result<(), chrono::ParseError> {
    NaiveTime::parse_from_str(text, TIME_FORMAT).map(|_| ())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub break_duration: DurationMinutes,
    pub break_interval: DurationMinutes,
    pub lunch_start: String,
    pub lunch_end: String,
}

impl Settings {
    pub fn init_config() -> Result<Settings> {
        let break_interval = Input::new()
            .with_prompt("Break interval")
            .default(55)
            .interact()?;

        let break_duration = Input::new()
            .with_prompt("Break duration")
            .default(5)
            .interact()?;

        let lunch_start = Input::new()
            .with_prompt("Lunch start")
            .default("11:30".to_string())
            .validate_with(validate_clock)
            .interact()?;

        let lunch_end = Input::new()
            .with_prompt("Lunch start")
            .default("12:30".to_string())
            .validate_with(validate_clock)
            .interact()?;

        let config = Settings {
            break_interval,
            break_duration,
            lunch_start,
            lunch_end,
        };
        return Ok(config);
    }
}

pub struct Service<'a, S>
where
    S: store::Store,
{
    store: &'a S,
}

const CONFIG_NAME: &str = "mob-config";

impl<'a, S> Service<'a, S>
where
    S: store::Store,
{
    pub fn new(store: &'a S) -> Self {
        Service { store }
    }

    pub fn load(&self) -> Result<Settings> {
        let res = self.store.load(CONFIG_NAME);
        match res {
            Ok(config) => Ok(config),
            Err(store::Error::Missing) => self.ask_to_create(),
            Err(error) => Err(Error::from(error)),
        }
    }

    fn save_retry(&self, config: &Settings) -> Result<()> {
        loop {
            match self.store.save(CONFIG_NAME, &config) {
                Ok(_) => return Ok(()),
                Err(store::Error::Conflict(error)) => {
                    println!("Saving config failed: {}", error);

                    let retry = dialoguer::Confirmation::new()
                        .with_text("Retry?")
                        .default(true)
                        .interact()?;

                    if !retry {
                        return Err(anyhow!("Could not save configuration"));
                    }
                }
                Err(error) => return Err(Error::from(error)),
            }
        }
    }

    fn ask_to_create(&self) -> Result<Settings> {
        let create = dialoguer::Confirmation::new()
            .with_text("This repo is not configured for mobbing, setup now?")
            .default(true)
            .interact()?;

        match create {
            true => {
                let c = Settings::init_config()?;
                self.save_retry(&c)?;
                Ok(c)
            }
            false => Err(Error::from(store::Error::Missing)),
        }
    }
}
