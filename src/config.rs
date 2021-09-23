use anyhow::{anyhow, Result};
use confy;
use dialoguer::{Confirm, Input};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::PathBuf;
use which::which;

const DEFAULT_REMOTE: &str = "origin";
const CONFIG_FILE: &str = ".mob";

pub const VAR_NEXT_DRIVER: &str = "NEXT_DRIVER";
pub const VAR_CURRENT_DRIVER: &str = "CURRENT_DRIVER";

const AFTER_TIMER_MESSAGE: &str = "mob next 'NEXT_DRIVER'";

#[derive(Serialize, Deserialize)]
pub struct Hooks {
    pub before_start: Option<String>,
    pub after_start: Option<String>,
    pub after_timer: Option<String>,
    pub before_next: Option<String>,
    pub after_next: Option<String>,
    pub before_done: Option<String>,
    pub after_done: Option<String>,
}

impl Hooks {
    pub fn new(after_timer: Option<String>) -> Self {
        Hooks {
            before_start: None,
            after_start: None,
            after_timer,
            before_next: None,
            after_next: None,
            before_done: None,
            after_done: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub remote: String,
    pub hooks: Hooks,
}

impl Config {
    pub fn ask() -> Result<Config> {
        log::info!("It seems like this is the first time you run mob. Welcome!");

        let name = Input::new()
            .with_prompt("Your name")
            .default(whoami::realname())
            .interact()?;

        let remote = Input::new()
            .with_prompt("Remote name you will use")
            .default(DEFAULT_REMOTE.to_string())
            .interact()?;

        let after_timer = ask_after_timer();
        let hooks = Hooks::new(after_timer);

        Ok(Config {
            name,
            remote,
            hooks,
        })
    }
}

fn ask_after_timer() -> Option<String> {
    let cmd = match after_timer_command() {
        Some(cmd) => cmd,
        None => return None,
    };

    log::info!("Command to run when your turn is done:");
    log::info!("  {}", &cmd);

    Confirm::new()
        .with_prompt("Add this to your config?")
        .default(true)
        .interact()
        .unwrap()
        .then(|| cmd)
}

fn after_timer_command() -> Option<String> {
    let cmd = [get_sound_command(), get_notify_command()]
        .iter()
        .filter_map(|c| c.clone())
        .collect::<Vec<String>>()
        .join(";");

    if cmd.is_empty() {
        return None;
    }
    Some(cmd)
}

fn get_sound_command() -> Option<String> {
    which("say")
        .map(format_cmd)
        .or_else(|_| {
            which("festival").map(|p| {
                format!(
                    "echo '{}' | {} --tts",
                    AFTER_TIMER_MESSAGE,
                    p.to_str().unwrap()
                )
            })
        })
        .or_else(|_| which("spd-say").map(format_cmd))
        .or_else(|_| which("espeak").map(format_cmd))
        .or_else(|_| which("beep").map(|p| p.into_os_string().into_string().unwrap()))
        .ok()
}

fn get_notify_command() -> Option<String> {
    which("osascript")
        .map(|p| {
            format!(
                r#"{} -e 'display notification "{}"'"#,
                p.to_str().unwrap(),
                AFTER_TIMER_MESSAGE
            )
        })
        .or_else(|_| which("notify-send").map(format_cmd))
        .ok()
}

fn format_cmd(path: PathBuf) -> String {
    format!("{} '{}'", path.to_str().unwrap(), AFTER_TIMER_MESSAGE)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            remote: DEFAULT_REMOTE.to_string(),
            hooks: Hooks::new(None),
        }
    }
}

pub fn load() -> Result<Config> {
    let path = {
        let user_dirs = UserDirs::new().unwrap();
        let home_dir = user_dirs.home_dir();
        home_dir.join(CONFIG_FILE)
    };
    let path_str = path.to_str().unwrap();

    let config: Config = confy::load_path(&path).map_err(|e| {
        anyhow!(
            "Failed to load config, check '{}' (or delete it to recreate): {}",
            &path_str,
            e
        )
    })?;

    if config.name.is_empty() {
        let config = Config::ask()?;
        confy::store_path(&path, &config)?;
        log::info!("Stored config to {}", &path_str);
        return Ok(config);
    }
    Ok(config)
}
