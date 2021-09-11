use super::v0::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionV0 {
    pub last_break: DateTime<Utc>,
    pub drivers: Drivers,
    pub branches: Branches,
    pub settings: Option<SettingsV0>,
    pub state: StateV0,
}
