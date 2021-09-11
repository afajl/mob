use serde::{Deserialize, Serialize};

type DurationMinutes = i64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsV0 {
    pub commit_message: String,
    pub work_duration: DurationMinutes,
    pub break_duration: DurationMinutes,
    pub break_interval: DurationMinutes,
    pub lunch_start: String,
    pub lunch_end: String,
}
