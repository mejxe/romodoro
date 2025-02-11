use std::{fmt:: Display,time::Duration};
use tokio_util::sync::CancellationToken;
use crate::{settings::*, DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PomodoroState {
    Work(i64),
    Break(i64)
}

#[derive(Debug, Clone)]
pub struct Timer {
     running: bool,
     time_left: i64,
     iteration: u8,
     total_iterations: u8,
     total_time: i64,
     total_elapsed: i64,
     work_state: PomodoroState,
     break_state: PomodoroState,
     current_state: PomodoroState,
     next_state: PomodoroState,
}
pub enum TimerCommand {
    Start,
    NextIteration,
    Customize(Settings),
    Stop,
}
impl Timer {
    pub fn get_duration(pomodoro_state: &PomodoroState) -> i64 {
        match pomodoro_state {
                PomodoroState::Work(dur) | PomodoroState::Break(dur) => {return *dur}
            };
    }
    pub fn set_total_time(&mut self) {
        let duration = Timer::get_duration(&self.work_state);
        self.total_time = duration * self.total_iterations as i64;
    }

     pub async fn run(&mut self, sender: tokio::sync::mpsc::Sender<i64>,mut command_rx: tokio::sync::mpsc::Receiver<TimerCommand>, close: CancellationToken) {
         loop {
             tokio::select! {
             Some(command) = command_rx.recv() => {
                 match command {
                     TimerCommand::Start => {self.running = true},
                     TimerCommand::Stop => {self.running = false},
                     TimerCommand::NextIteration => {self.next_iteration()}
                     TimerCommand::Customize(setting) => self.set_setting(setting),
                 };
             }
             _ = tokio::time::sleep(Duration::from_secs(1)), if self.running && self.time_left >= 0 => {
                 self.time_left -= 1;
                 if let PomodoroState::Work(_) = self.current_state {
                     self.total_elapsed += 1;
                 }
                 sender.send(self.time_left).await.unwrap();
             }
             _ = close.cancelled() => {break}
             }
         }
     }

    pub fn swap_states(&mut self) {
        if !self.running {
            match self.current_state {
                PomodoroState::Work(_) => {
                    self.current_state = self.break_state;
                    self.next_state = self.work_state;
                },
                PomodoroState::Break(_) => {
                    self.current_state = self.work_state;
                    self.next_state = self.break_state;
                },
            }
            let duration = Timer::get_duration(&self.current_state);
            self.time_left = duration;
        }

    }
    pub fn next_iteration(&mut self) {
        self.swap_states();
        if let PomodoroState::Work(_) = self.current_state {
            self.iteration += 1;
        }
    }
    pub fn set_setting(&mut self, setting: Settings) {
        if self.running {return};
        self.restart();
        match setting {
            Settings::Iterations(iterations) => self.total_iterations = iterations.unwrap(),
            Settings::WorkTime(_) =>{self.current_state = PomodoroState::from(setting); self.work_state = PomodoroState::from(setting)},
            Settings::BreakTime(_) => self.next_state = PomodoroState::from(setting),
        }
        self.restart();
    }


    pub fn restart(&mut self) {
        if let PomodoroState::Break(_) = self.current_state {
            self.swap_states();
        }
        self.iteration = 1;
        self.time_left = Timer::get_duration(&self.work_state);
        self.total_elapsed = 0;
        self.running = false;
        self.set_total_time();
    }
    pub fn get_timeleft(&self) -> i64 {
        self.time_left
    }
    pub fn get_work_state(&self) -> PomodoroState {
        self.work_state
    }

    pub fn get_break_state(&self) -> PomodoroState {
        self.break_state
    }

    pub fn get_running(&self) -> bool {
         self.running
    }
    pub fn get_total_iterations(&self) -> u8 {
        self.total_iterations
    }
    pub fn get_iteration(&self) -> u8 {
        self.iteration
    }
    pub fn get_total_time(&self) -> i64 {
        self.total_time
    }
    pub fn get_total_elapsed_time(&self) -> i64 {
        self.total_elapsed
    }
    pub fn get_current_state(&self) -> PomodoroState {
        self.current_state
    }
    pub fn set_running(&mut self, state: bool) {
        self.running = state;
    }
    pub fn set_elapsed_time(&mut self, elapsed: i64) {
        self.total_elapsed = elapsed;
    }
    pub fn set_time_left(&mut self, time: i64) {
        self.time_left = time;
    }
}
// traits
impl Into<Settings> for &PomodoroState {
    fn into(self) -> Settings {
        match self {
            PomodoroState::Break(time) => { Settings::BreakTime(Some(*time)) },
            PomodoroState::Work(time) => Settings::BreakTime(Some(*time)) }

    }
}
impl From<Settings> for PomodoroState {
    fn from(value: Settings) -> Self {
        match value {
            Settings::WorkTime(Some(time)) => PomodoroState::Work(time),
            Settings::BreakTime(Some(time)) => PomodoroState::Break(time),
            _ => PomodoroState::Break(-1),
        }

    }
}

impl Display for PomodoroState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PomodoroState::Work(_) => {
                write!(f, "Work")
            }
            PomodoroState::Break(_) => {
                write!(f, "Break")
            }
        }
    }
}
impl Default for Timer {
    fn default() -> Self {
        let work_state = PomodoroState::Work(DEFAULT_WORK);
        let break_state = PomodoroState::Break(DEFAULT_BREAK);
        let total_iterations = DEFAULT_ITERATIONS;
        let duration = Timer::get_duration(&work_state);
        let total_time: i64 = duration * total_iterations as i64;
        Timer { running: false, total_iterations, current_state: work_state, time_left:duration , next_state:break_state, iteration: 1, total_time, total_elapsed: 0, work_state, break_state}
        
    }
}

#[cfg(test)]
mod tests {


    use super::*;

}
