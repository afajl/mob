use super::session::Session;
use super::session_v0::SessionV0;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
// Minimal struct to get version
pub struct Versioned {
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedSession {
    #[serde(rename = "v0")]
    V0(SessionV0),

    #[serde(rename = "v1")]
    V1(Session),
}

impl VersionedSession {
    fn migrate(self) -> Self {
        match self {
            VersionedSession::V0(session) => VersionedSession::V1(Session::from(session)),
            VersionedSession::V1(_) => self,
        }
    }

    pub fn latest(self) -> Session {
        let mut version = self.migrate();
        loop {
            if let VersionedSession::V1(session) = version {
                return session;
            }
            version = version.migrate();
        }
    }
}
