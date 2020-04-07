use anyhow::Result;
use confy;
use serde::{Deserialize, Serialize};

const CONFIG_NAME: &str = "mob";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub say_command: String,
    pub notify_command: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            name: whoami::user(),
            say_command: "say '{}'".into(),
            notify_command: "/usr/bin/osascript -e 'display notification \"{}\"'".into(),
        }
    }
}

pub fn load() -> Result<Config> {
    confy::load(CONFIG_NAME).map_err(anyhow::Error::from)
}
