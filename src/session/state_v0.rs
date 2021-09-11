use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateV0 {
    Stopped,
    Working {
        driver: String,
    },
    WaitingForNext {
        next: Option<String>,
        is_break: bool,
    },
}
