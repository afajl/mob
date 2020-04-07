use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

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

pub enum Key {
    Settings,
    Session,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Key::Settings => write!(f, "settings"),
            Key::Session => write!(f, "session"),
        }
    }
}

pub trait Store<T>
where
    T: DeserializeOwned + ?Sized + Serialize,
{
    fn load(&self, key: Key) -> Result<T, Error>;
    fn save(&self, key: Key, data: &T) -> Result<(), Error>;
}
