use std::{fs};

use crate::{timer::*, DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK, WORK_TIME_INCR, BREAK_TIME_INCR};
use serde::*;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsTab {
    pub selected_setting: usize,
    pub ui_settings: UISettings,
    pub timer_settings: TimerSettings,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub pause_after_state_change: bool,
    pub hide_work_countdown: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSettings {
    pub work_time : i64,
    pub break_time : i64,
    pub iterations : u8,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PomodoroSettings {
    WorkTime(Option<i64>),
    BreakTime(Option<i64>),
    Iterations(Option<u8>),
}
        
        
impl SettingsTab {
    pub fn save_to_file(&self) {
        let path = ProjectDirs::from("romodoro","mejxedev", "romodoro").unwrap();
        let path = path.config_dir();
        if !path.exists() {fs::create_dir(&path).unwrap();}
        let toml_cfg: String = toml::to_string(&self).unwrap();
        fs::write(path.join("config.toml"), toml_cfg).unwrap();
    }
    pub fn new() -> SettingsTab {
        let path = ProjectDirs::from("romodoro","mejxedev", "romodoro").unwrap();
        let path = path.config_dir();
        let config: SettingsTab = toml::from_str(&fs::read_to_string(path.join("config.toml")).unwrap_or("".to_string())).unwrap_or(SettingsTab::default());
        config
    }
    pub fn restore_defaults(&mut self) {
        self.timer_settings = TimerSettings::default();
        self.ui_settings = UISettings::default();
    }

    pub fn get_pomodoro_setting(&self, setting:PomodoroSettings) -> PomodoroSettings {
        match setting {
            PomodoroSettings::BreakTime(_) => {
                PomodoroSettings::BreakTime(Some(self.timer_settings.break_time))
            }
            PomodoroSettings::WorkTime(_) => {
                PomodoroSettings::WorkTime(Some(self.timer_settings.work_time))
            }
            PomodoroSettings::Iterations(_) => {
                PomodoroSettings::Iterations(Some(self.timer_settings.iterations))
            }

        }
    }

    pub fn select_down(&mut self) {
        if self.selected_setting == 4 {
            self.selected_setting = 0;
        } else { self.selected_setting += 1}
    }
    pub fn select_up(&mut self) {
        if self.selected_setting == 0 {
            self.selected_setting = 4;
        } else { self.selected_setting -= 1}
    }
    pub fn decrement(&mut self) {
        match self.selected_setting {
            0 if self.timer_settings.work_time - WORK_TIME_INCR != 0 => {self.timer_settings.work_time -= WORK_TIME_INCR},
            1 if self.timer_settings.break_time - BREAK_TIME_INCR != 0  => {self.timer_settings.break_time -= BREAK_TIME_INCR},
            2 if self.timer_settings.iterations - 1 > 0 => {self.timer_settings.iterations -= 1},
            3 => self.ui_settings.pause_after_state_change = !self.ui_settings.pause_after_state_change,
            4 => self.ui_settings.hide_work_countdown = !self.ui_settings.hide_work_countdown,
            _ => {},
        }
    }
    pub fn increment(&mut self) {
        match self.selected_setting {
            0 => {self.timer_settings.work_time += WORK_TIME_INCR},
            1 => {self.timer_settings.break_time += BREAK_TIME_INCR},
            2 => {self.timer_settings.iterations += 1},
            3 => self.ui_settings.pause_after_state_change = !self.ui_settings.pause_after_state_change,
            4 => self.ui_settings.hide_work_countdown = !self.ui_settings.hide_work_countdown,
            _ => {}
        }
    }
}
impl Default for UISettings {
    fn default() -> Self {
        UISettings { pause_after_state_change: false, hide_work_countdown: false}
    }
}
impl Default for TimerSettings {
    fn default() -> Self {
        TimerSettings {work_time: DEFAULT_WORK, break_time: DEFAULT_BREAK, iterations: DEFAULT_ITERATIONS}
    }
}
impl From<PomodoroState> for PomodoroSettings {
    fn from(value: PomodoroState) -> Self {
        match value {
            PomodoroState::Work(time) => PomodoroSettings::WorkTime(Some(time)),
            PomodoroState::Break(time) => PomodoroSettings::BreakTime(Some(time)),
        }
    }
}
impl From<u8> for PomodoroSettings {
    fn from(value: u8) -> Self {
        PomodoroSettings::Iterations(Some(value))
    }
}
#[cfg(test)]
mod tests {
    use super::SettingsTab;

    #[test]
    fn save_to_file() {
        let settings = SettingsTab::default();
        settings.save_to_file();
    }
    #[test]
    fn read() {
        let settings = SettingsTab::new();
        dbg!(settings);
    }
}

