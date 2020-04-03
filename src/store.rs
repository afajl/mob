use serde::{de::DeserializeOwned, Serialize};

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
    fn load<T: DeserializeOwned>(&self, name: &str) -> Result<T, Error>;
    fn save<T: ?Sized + Serialize>(&self, name: &str, data: &T) -> Result<(), Error>;
}
