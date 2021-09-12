use anyhow::Result;
use confy;
use dialoguer::{Confirm, Input};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path;

const CONFIG_FILE: &str = ".mob";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub remote: String,
    pub say_command: Option<String>,
    pub notify_command: Option<String>,
}

impl Config {
    pub fn ask() -> Result<Config> {
        log::info!("It seems like this is the first time you run mob. Welcome!");

        let default = Config::default();

        let name = Input::new()
            .with_prompt("Your name")
            .default(whoami::realname())
            .interact()?;

        let remote = Input::new()
            .with_prompt("Remote name you will use")
            .default(default.remote)
            .interact()?;

        let use_say_comand = Confirm::new()
            .with_prompt("Do you want to use speech synthesis for prompts?")
            .default(true)
            .interact()?;

        let say_command = if use_say_comand {
            Some(
                Input::new()
                    .with_prompt("Command to say something on your computer")
                    .default(default.say_command.unwrap())
                    .interact()?,
            )
        } else {
            None
        };

        let use_notify_command = Confirm::new()
            .with_prompt("Do you want to show desktop notifications?")
            .default(true)
            .interact()?;

        let notify_command = if use_notify_command {
            Some(
                Input::new()
                    .with_prompt("Command to notify you (empty input will disable)")
                    .default(default.notify_command.unwrap())
                    .interact()?,
            )
        } else {
            None
        };

        Ok(Config {
            name,
            remote,
            say_command,
            notify_command,
        })
    }
    pub fn commands(&self) -> Vec<String> {
        vec![self.say_command.clone(), self.notify_command.clone()]
            .into_iter()
            .flatten()
            .collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            //name: whoami::user(),
            name: "".to_string(),
            remote: "origin".to_string(),
            say_command: Some("say 'MESSAGE'".into()),
            notify_command: Some("/usr/bin/osascript -e 'display notification \"MESSAGE\"'".into()),
        }
    }
}

pub fn load() -> Result<Config> {
    let config: Config = confy::load_path(config_path()).map_err(anyhow::Error::from)?;
    if config.name.is_empty() {
        let config = Config::ask()?;
        confy::store_path(config_path(), &config)?;
        log::info!("Stored config to {}", config_path().to_str().unwrap());
        return Ok(config);
    }
    Ok(config)
}

fn config_path() -> path::PathBuf {
    let user_dirs = UserDirs::new().unwrap();
    let home_dir = user_dirs.home_dir();
    home_dir.join(CONFIG_FILE)
}
