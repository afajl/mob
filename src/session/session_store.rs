use super::latest::Session;
use super::versioned_session::Versioned;
use super::VersionedSession;
use crate::git;
use crate::session::v0::SessionV0;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("git error: `{0}`")]
    Git(#[from] git::store::Error),

    #[error("unable to deserialize session `{0}`, run `mob clean`")]
    Format(#[from] serde_json::Error),

    #[error("you're version is to old, upgrade to version `{0}`")]
    NewerVersion(String),
}

pub trait Store {
    fn load(&self) -> Result<Session>;
    fn save(&self, session: Session) -> Result<()>;
    fn clean(&self) -> Result<()>;
}

pub struct SessionStore<'a> {
    store: &'a dyn git::Store,
}

impl<'a> SessionStore<'a> {
    pub fn new(store: &'a impl git::Store) -> Self {
        SessionStore { store }
    }

    fn get_session(data: Vec<u8>) -> Result<Session> {
        match serde_json::from_slice::<VersionedSession>(data.as_slice()) {
            Ok(versioned_session) => Ok(versioned_session.latest()),
            Err(..) => {
                let versioned = serde_json::from_slice::<Versioned>(data.as_slice())?;
                match versioned.version {
                    Some(version) => Err(Error::NewerVersion(version)),
                    None => {
                        // Assume first unversioned
                        let session = serde_json::from_slice::<SessionV0>(data.as_slice())?;
                        Ok(VersionedSession::V0(session).latest())
                    }
                }
            }
        }
    }
}

impl<'a> Store for SessionStore<'a> {
    fn load(&self) -> Result<Session> {
        match self.store.load() {
            Ok(data) => SessionStore::get_session(data),
            Err(git::store::Error::Missing) => Ok(Session::default()),
            Err(error) => Err(Error::Git(error)),
        }
    }

    fn save(&self, session: Session) -> Result<()> {
        let versioned_session = VersionedSession::V1(session);
        let json = serde_json::to_vec_pretty(&versioned_session)?;
        self.store.save(&json)?;
        Ok(())
    }

    fn clean(&self) -> Result<()> {
        self.store.clean()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git;

    struct MockStore {
        pub load_data: Vec<u8>,
    }

    impl git::Store for MockStore {
        fn load(&self) -> std::result::Result<Vec<u8>, git::store::Error> {
            Ok(self.load_data.clone())
        }
        fn clean(&self) -> std::result::Result<(), git::store::Error> {
            todo!()
        }
        fn save(&self, _: &[u8]) -> std::result::Result<(), git::store::Error> {
            todo!()
        }
    }

    #[test]
    fn load_invalid_data() {
        let store = MockStore {
            load_data: Vec::from("invalid: 'json"),
        };

        let session_store = SessionStore::new(&store);

        if session_store.load().is_ok() {
            panic!("We should fail")
        }
    }

    #[test]
    fn newer_version() {
        let json = r#"{
                  "version": "v99",
                  "drivers": [],
                  "settings": {
                    "commit_message": "mob sync [skip ci]"
                  },
                  "state": {
                    "WaitingForNext": {
                      "next": "Johan Rydenstam",
                      "is_break": false
                    }
                  }
                }"#;
        let store = MockStore {
            load_data: Vec::from(json),
        };

        let session_store = SessionStore::new(&store);

        match session_store.load() {
            Err(Error::NewerVersion(version)) => assert_eq!(version, "v99"),
            other => panic!("Should fail but got {:?}", other),
        }
    }

    #[test]
    fn unversioned() {
        let json = r#"{
                  "drivers": [],
                  "last_break": "2021-09-10T14:22:41.083716Z",
                  "branches": {
                    "branch": "mob-session",
                    "base_branch": "main"
                  },
                  "settings": {
                    "commit_message": "mob sync [skip ci]",
                    "work_duration": 10,
                    "break_duration": 5,
                    "break_interval": 55,
                    "lunch_start": "11:30",
                    "lunch_end": "12:30"
                  },
                  "state": {
                    "WaitingForNext": {
                      "next": "Johan Rydenstam",
                      "is_break": false
                    }
                  }
                }"#;
        let store = MockStore {
            load_data: Vec::from(json),
        };

        let session_store = SessionStore::new(&store);

        if let Err(err) = session_store.load() {
            panic!("Got error but expected oldest version: {:?}", err)
        }
    }
}
