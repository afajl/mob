use crate::prompt::Prompter;
use anyhow::Result;
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
    pub fn ask(prompter: &dyn Prompter, default: Branches) -> Result<Branches> {
        if default.base_branch != "master" && default.base_branch != "main" {
            log::info!("Note that you are not on main or master");
        }

        let base_branch = prompter.input_string("Base branch", &default.base_branch)?;

        let branch = prompter.input_string("Feature branch", &default.branch)?;

        Ok(Branches {
            branch,
            base_branch,
        })
    }
}

impl Default for Branches {
    fn default() -> Self {
        Self {
            base_branch: "main".to_string(),
            branch: "mob-session".to_string(),
        }
    }
}
