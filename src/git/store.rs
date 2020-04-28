use super::*;
use crate::session::Session;

const SESSION_FILENAME: &'static str = "data";
const SESSION_HEAD: &'static str = "mob-meta";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failure pushing to origin: `{0}`")]
    Conflict(#[from] anyhow::Error),

    #[error("unable to deserialize data `{0}`")]
    Format(#[from] serde_json::Error),

    #[error("unknown error")]
    Other(#[from] git2::Error),

    #[error("missing config")]
    Missing,
}

pub trait Store {
    fn load(&self) -> Result<Session, Error>;
    fn save(&self, data: &Session) -> Result<(), Error>;
    fn clean(&self) -> Result<(), Error>;
}

impl<'repo> Store for GitCommand<'repo> {
    fn save(&self, data: &Session) -> Result<(), store::Error> {
        let json = serde_json::to_vec(data)?;

        let filename = SESSION_FILENAME;

        let commit = CommitFile {
            filename,
            data: json.as_slice(),
            reference: SESSION_HEAD,
            message: "save",
        };

        self.create_commit(commit)?;

        self.run_quietly(&[
            "push",
            self.remote.as_str(),
            format!("{}:{}", SESSION_HEAD, SESSION_HEAD).as_str(),
        ])
        .map_err(store::Error::Conflict) // TODO: should check for "rejected" in output
    }

    fn load(&self) -> Result<Session, store::Error> {
        if let Err(err) = self.run_quietly(&[
            "fetch",
            self.remote.as_str(),
            format!("{}:{}", SESSION_HEAD, SESSION_HEAD).as_str(),
        ]) {
            log::trace!(
                "Could not fetch remote mob branch {}: {}",
                SESSION_HEAD,
                err
            );
        }

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
        let blob = blob.content();

        serde_json::from_slice(blob).map_err(store::Error::Format)
    }

    fn clean(&self) -> Result<(), store::Error> {
        self.run_quietly(&["branch", "-D", SESSION_HEAD])
            .unwrap_or_else(|err| log::trace!("Failed to delete local branch: {}", err));
        self.run_quietly(&["push", self.remote.as_str(), "--delete", SESSION_HEAD])
            .unwrap_or_else(|err| log::trace!("Failed to remove remote branch: {}", err));
        Ok(())
    }
}
