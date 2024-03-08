use anyhow::anyhow;

use super::{store, CommitFile, GitCommand, Result};

const SESSION_FILENAME: &str = "data";
const SESSION_HEAD: &str = "mob-meta";
const COMMIT_MESSAGE: &str = "mob metadata changed [skip ci]";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("unable to deserialize data `{0}`")]
    // Format(#[from] serde_json::Error),
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

impl<'repo> Store for GitCommand<'repo> {
    fn save(&self, data: &[u8]) -> Result<(), store::Error> {
        let commit = CommitFile {
            filename: SESSION_FILENAME,
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
        self.run_quietly(&["branch", "-D", SESSION_HEAD])
            .map_err(|err| Error::Missing {
                source: err.context("Could not delete local meta branch: {SESSION_HEAD}"),
            })?;

        self.run_quietly(&[
            "fetch",
            self.remote.as_str(),
            format!("{}:{}", SESSION_HEAD, SESSION_HEAD).as_str(),
        ])
        .map_err(|err| Error::Missing {
            source: err.context("Could not fetch repo"),
        })?;

        let commit = self.last_commit(SESSION_HEAD);

        let commit = match commit {
            Some(commit) => commit,
            None => {
                return Err(store::Error::Missing {
                    source: anyhow!("Could not find last commit"),
                })
            }
        };

        let tree = commit.tree()?;

        let tree_entry = tree.get_name(SESSION_FILENAME);

        let tree_entry = match tree_entry {
            Some(tree) => tree,
            None => {
                return Err(store::Error::Missing {
                    source: anyhow!("Could not find tree"),
                })
            }
        };

        let object = tree_entry.to_object(&self.repo)?;
        let blob = object.as_blob();
        let blob = match blob {
            Some(blob) => blob,
            None => {
                return Err(store::Error::Missing {
                    source: anyhow!("No blob"),
                })
            }
        };
        Ok(blob.content().into())
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
