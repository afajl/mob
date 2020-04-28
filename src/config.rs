use anyhow::Result;
use confy;
use dialoguer::Input;
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::path;

const CONFIG_FILE: &str = ".mob";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub remote: String,
    pub say_command: String,
    pub notify_command: String,
}

impl Config {
    pub fn ask() -> Result<Config> {
        log::info!("It seems like this is the first time you run mob. Welcome!");
        log::info!("I need some info:");

        let default = Config::default();

        let name = Input::new()
            .with_prompt("Your name")
            .default(whoami::user())
            .interact()?;

        let remote = Input::new()
            .with_prompt("Remote name you will use")
            .default(default.remote)
            .interact()?;

        let say_command = Input::new()
            .with_prompt("Command to say something on your computer (empty input will disable)")
            .default(default.say_command)
            .interact()?;

        let notify_command = Input::new()
            .with_prompt("Command to notify you (empty input will disable)")
            .default(default.notify_command)
            .interact()?;

        Ok(Config {
            name,
            remote,
            say_command,
            notify_command,
        })
    }
    pub fn commands(&self) -> Vec<String> {
        // TODO there must be a better way than cloning twice
        return [self.say_command.clone(), self.notify_command.clone()]
            .iter()
            .filter(|c| c.len() > 0)
            .map(|c| c.clone())
            .collect();
    }
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            //name: whoami::user(),
            name: "".to_string(),
            remote: "origin".to_string(),
            say_command: "say 'MESSAGE'".into(),
            notify_command: "/usr/bin/osascript -e 'display notification \"MESSAGE\"'".into(),
        }
    }
}

pub fn load() -> Result<Config> {
    let config: Config = confy::load_path(config_path()).map_err(anyhow::Error::from)?;
    if config.name == "" {
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
