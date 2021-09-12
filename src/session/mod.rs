mod branches;
mod drivers;
#[allow(clippy::module_inception)]
mod session;
mod session_store;
mod session_v0;
mod settings;
mod settings_v0;
mod state;
mod state_v0;
mod versioned_session;

pub mod v0 {
    use super::*;
    pub use branches::Branches;
    pub use drivers::Drivers;
    pub use session_v0::SessionV0;
    pub use settings_v0::SettingsV0;
    pub use state_v0::StateV0;
}

pub mod latest {
    use super::*;
    pub use branches::Branches;
    pub use drivers::Drivers;
    pub use session::Session;
    pub use settings::Settings;
    pub use state::State;
}

pub use latest::*;
pub use session_store::{SessionStore, Store};
pub use versioned_session::VersionedSession;
