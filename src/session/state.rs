use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum State {
    Stopped,
    Working { driver: String },
    WaitingForNext { next: Option<String> },
}
