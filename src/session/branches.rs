use anyhow::Result;
use dialoguer::Input;
use serde::{Deserialize, Serialize};

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
        let default_base_branch = Branches::default().base_branch;
        if default.base_branch != default_base_branch {
            log::info!("Note that you are not on {}", default_base_branch)
        }

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
