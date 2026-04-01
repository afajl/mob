use anyhow::anyhow;

use super::{CommitFile, GitCommand, Result, store};

const SESSION_FILENAME: &str = "data";
const SESSION_HEAD: &str = "mob-meta";
const COMMIT_MESSAGE: &str = "mob metadata changed [skip ci]";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),

    #[error("failed to push to origin")]
    Conflict,

    #[error("missing config")]
    Missing { source: anyhow::Error },
}

pub trait Store {
    fn load(&self) -> Result<Vec<u8>, Error>;
    fn save(&self, data: &[u8]) -> Result<(), Error>;
    fn clean(&self) -> Result<(), Error>;
}

impl Store for GitCommand {
    fn save(&self, data: &[u8]) -> Result<(), store::Error> {
        let filename = SESSION_FILENAME;

        let commit = CommitFile {
            filename,
            data,
            reference: SESSION_HEAD,
            message: COMMIT_MESSAGE,
        };

        self.create_commit(&commit)?;

        self.run_quietly(&[
            "push",
            "--no-verify",
            self.remote.as_str(),
            format!("{}:{}", SESSION_HEAD, SESSION_HEAD).as_str(),
        ])
        .map_err(|_| store::Error::Conflict) // TODO: should check for "rejected" in output
    }

    fn load(&self) -> Result<Vec<u8>, store::Error> {
        if let Err(e) = self.run_quietly(&["branch", "-D", SESSION_HEAD]) {
            log::trace!("Failed to delete or missing local branch: {}", e);
        }

        self.run_quietly(&[
            "fetch",
            self.remote.as_str(),
            format!("{}:{}", SESSION_HEAD, SESSION_HEAD).as_str(),
        ])
        .map_err(|err| Error::Missing {
            source: err.context("Could not fetch repo"),
        })?;

        // Use git show to read the file content from the branch
        self.show_file(SESSION_HEAD, SESSION_FILENAME)
            .map_err(|err| Error::Missing {
                source: anyhow!("Could not read session data: {}", err),
            })
    }

    fn clean(&self) -> Result<(), store::Error> {
        self.run_quietly(&["branch", "-D", SESSION_HEAD])
            .unwrap_or_else(|err| log::trace!("Failed to delete local branch: {}", err));
        self.run_quietly(&[
            "push",
            self.remote.as_str(),
            "--no-verify",
            "--delete",
            SESSION_HEAD,
        ])
        .unwrap_or_else(|err| log::trace!("Failed to remove remote branch: {}", err));
        Ok(())
    }
}
