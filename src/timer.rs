use std::{fmt::{write, Display}, io::Write, ops::Mul, sync::Arc, time::Duration};
use tokio::{sync, io::AsyncWriteExt};
use tokio_util::sync::CancellationToken;

#[derive(Debug, PartialEq, Clone)]
pub enum PomodoroState {
    Work(i64),
    Break(i64)
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

#[derive(Debug, Clone)]
pub struct Timer {
     running: bool,
     time_left: i64,
     current_state: PomodoroState,
     next_state: PomodoroState,
     iteration: u8,
     total_iterations: u8,
     total_time: i64,
}
pub enum TimerCommand {
    Start,
    ChangeState,
    Stop,
}
impl Timer {
    pub fn new(work_state: PomodoroState, break_state: PomodoroState, total_iterations: u8) -> Self { 
        let duration = Timer::get_duration(&work_state);
        let total_time: i64 = duration * total_iterations as i64;
        Timer { running: false, total_iterations, current_state: work_state, time_left:duration , next_state:break_state, iteration: 0, total_time}
    }
    pub fn get_duration(pomodoro_state: &PomodoroState) -> i64 {
        match pomodoro_state {
                PomodoroState::Work(dur) | PomodoroState::Break(dur) => {return *dur}
            };
    }
     pub async fn run(&mut self, sender: tokio::sync::mpsc::Sender<i64>,mut command_rx: tokio::sync::mpsc::Receiver<TimerCommand>, close: CancellationToken) {
         loop {
             tokio::select! {
             Some(command) = command_rx.recv() => {
                 match command {
                     TimerCommand::Start => {self.running = true},
                     TimerCommand::Stop => {self.running = false},
                     TimerCommand::ChangeState => {self.running = true},
                 };
             }
             _ = tokio::time::sleep(Duration::from_secs(1)), if self.running && self.time_left > 0 => {
                 self.time_left -= 1;
                 sender.send(self.time_left).await.unwrap();
             }
             _ = close.cancelled() => {break}
             }
         }
     }

    pub fn next_state(&mut self) {
        if !self.running {
            std::mem::swap(&mut self.current_state, &mut self.next_state);
            let duration = Timer::get_duration(&self.current_state);
            self.time_left = duration;
        }

    }
    pub fn get_timeleft(&self) -> i64 {
        self.time_left
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
    pub fn get_work_state(&self) -> String {
        self.current_state.to_string()
    }
    pub fn set_running(&mut self, state: bool) {
        self.running = state;
    }
    pub fn set_time_left(&mut self, time: i64) {
        self.time_left = time;
    }
}

#[cfg(test)]
mod tests {

    use tokio::sync::mpsc;

    use super::*;
    #[test]
    fn next_state_works() {
        let mut timer = Timer::new(PomodoroState::Work(5), PomodoroState::Break(2), 4);
        timer.next_state();
        assert!(timer.current_state == PomodoroState::Break(2))
    }
   // #[tokio::test]
   // async fn full_test() {
   //     let mut timer = Timer::new(PomodoroState::Work(5), PomodoroState::Break(2), 4);
   //     let (tx, mut rx) = mpsc::channel(4);
   //     timer.set_running(true);
   //     let timer1 = Arc::new(tokio::sync::Mutex::new(timer.clone()));
   //     let task = tokio::task::spawn(async move {timer1.lock().await.run(tx).await});
   //     tokio::time::sleep(Duration::from_secs(5)).await;
   //     assert_eq!(time, 0);
   // }
    #[test]
    fn api_works() {
        let mut timer = Timer::new(PomodoroState::Work(5), PomodoroState::Break(2), 4);
        assert!(timer.get_work_state() == "Work".to_string())
    }

}
