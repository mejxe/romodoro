use serde::{Deserialize, Serialize};

const POMODORO_SETTINGS: u8 = 6;
const STATS_SETTINGS: u8 = 3;
#[derive(Debug, Clone, Deserialize, Serialize, Default, Copy, PartialEq)]
pub enum SettingsTabs {
    #[default]
    Pomodoro,
    Stats,
}
impl SettingsTabs {
    pub fn amount(&self) -> u8 {
        match self {
            Self::Pomodoro => POMODORO_SETTINGS,
            Self::Stats => STATS_SETTINGS,
        }
    }
}
