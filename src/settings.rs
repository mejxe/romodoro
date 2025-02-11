use crate::{timer::*, DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK, WORK_TIME_INCR, BREAK_TIME_INCR};


#[derive(Debug, Clone)]
pub struct SettingsTab {
    pub selected_setting: usize,
    pub work_time : i64,
    pub break_time : i64,
    pub iterations : u8,
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Settings {
    WorkTime(Option<i64>),
    BreakTime(Option<i64>),
    Iterations(Option<u8>),
}
impl From<PomodoroState> for Settings {
    fn from(value: PomodoroState) -> Self {
        match value {
            PomodoroState::Work(time) => Settings::WorkTime(Some(time)),
            PomodoroState::Break(time) => Settings::BreakTime(Some(time)),
        }
    }
}
impl From<u8> for Settings {
    fn from(value: u8) -> Self {
        Settings::Iterations(Some(value))
    }
}
        
        
impl SettingsTab {
    pub fn get_setting(&self, setting:Settings) -> Settings {
        match setting {
            Settings::BreakTime(_) => {
                Settings::BreakTime(Some(self.break_time))
            }
            Settings::WorkTime(_) => {
                Settings::WorkTime(Some(self.work_time))
            }
            Settings::Iterations(_) => {
                Settings::Iterations(Some(self.iterations))
            }

        }
    }

    pub fn select_down(&mut self) {
        if self.selected_setting == 2 {
            self.selected_setting = 0;
        } else { self.selected_setting += 1}
    }
    pub fn select_up(&mut self) {
        if self.selected_setting == 0 {
            self.selected_setting = 2;
        } else { self.selected_setting -= 1}
    }
    pub fn decrement(&mut self) {
        match self.selected_setting {
            0 if self.work_time - WORK_TIME_INCR != 0 => {self.work_time -= WORK_TIME_INCR},
            1 if self.break_time - BREAK_TIME_INCR != 0  => {self.break_time -= BREAK_TIME_INCR},
            2 => {self.iterations -= 1},
            _ => {},
        }
    }
    pub fn increment(&mut self) {
        match self.selected_setting {
            0 => {self.work_time += WORK_TIME_INCR},
            1 => {self.break_time += BREAK_TIME_INCR},
            2 => {self.iterations += 1},
            _ => {}
        }
    }
}
impl Default for SettingsTab {
    fn default() -> Self {
        SettingsTab { selected_setting: 0, work_time: DEFAULT_WORK, break_time: DEFAULT_BREAK, iterations: DEFAULT_ITERATIONS }
    }
}
