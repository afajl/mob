use super::*;

const SESSION_FILENAME: &str = "data";
const SESSION_HEAD: &str = "mob-meta";
const COMMIT_MESSAGE: &str = "mob metadata changed [skip ci]";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failure pushing to origin: `{0}`")]
    Conflict(#[from] anyhow::Error),

    // #[error("unable to deserialize data `{0}`")]
    // Format(#[from] serde_json::Error),
    #[error("unknown error")]
    Other(#[from] git2::Error),

    #[error("missing config")]
    Missing,
}

pub trait Store {
    fn load(&self) -> Result<Vec<u8>, Error>;
    fn save(&self, data: &[u8]) -> Result<(), Error>;
    fn clean(&self) -> Result<(), Error>;
}

impl<'repo> Store for GitCommand<'repo> {
    fn save(&self, data: &[u8]) -> Result<(), store::Error> {
        let filename = SESSION_FILENAME;

        let commit = CommitFile {
            filename,
            data,
            reference: SESSION_HEAD,
            message: COMMIT_MESSAGE,
        };

        self.create_commit(commit)?;

        self.run_quietly(&[
            "push",
            "--no-verify",
            self.remote.as_str(),
            format!("{}:{}", SESSION_HEAD, SESSION_HEAD).as_str(),
        ])
        .map_err(store::Error::Conflict) // TODO: should check for "rejected" in output
    }

    fn load(&self) -> Result<Vec<u8>, store::Error> {
        self.run_quietly(&["branch", "-D", SESSION_HEAD])
            .unwrap_or_else(|err| {
                log::trace!(
                    "Could not delete local mob branch {}: {}",
                    SESSION_HEAD,
                    err
                )
            });

        self.run_quietly(&[
            "fetch",
            self.remote.as_str(),
            format!("{}:{}", SESSION_HEAD, SESSION_HEAD).as_str(),
        ])
        .unwrap_or_else(|err| {
            log::trace!(
                "Could not fetch remote mob branch {}: {}",
                SESSION_HEAD,
                err
            )
        });

        let commit = self.last_commit(SESSION_HEAD);

        let commit = match commit {
            Some(commit) => commit,
            None => return Err(store::Error::Missing),
        };

        let tree = commit.tree()?;

        let tree_entry = tree.get_name(SESSION_FILENAME);

        let tree_entry = match tree_entry {
            Some(tree) => tree,
            None => return Err(store::Error::Missing),
        };

        let object = tree_entry.to_object(&self.repo)?;
        let blob = object.as_blob();
        let blob = match blob {
            Some(blob) => blob,
            None => return Err(store::Error::Missing),
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
