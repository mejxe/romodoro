use tokio_util::sync::CancellationToken;

use crate::{app::Event, settings::Settings, timer::*};

#[derive(Debug)]
pub struct Pomodoro {
    pub timer : Timer,
    time_sender: tokio::sync::mpsc::Sender<i64>,
    command_rx: Option<tokio::sync::mpsc::Receiver<TimerCommand>>,
}
impl Pomodoro {
    pub fn new(time_sender: tokio::sync::mpsc::Sender<i64>, command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,command_tx: tokio::sync::mpsc::Sender<TimerCommand>) -> Self {
        let mut timer = Timer::default();
        timer.countdown_command_tx = Some(command_tx);
        Pomodoro {timer, command_rx: Some(command_rx), time_sender}
    }
    pub async fn create_countdown(&mut self, cancel_token: CancellationToken) { 
        let sender = self.time_sender.clone();
        let command_rx = self.command_rx.take().expect("Timer initialized");
        self.timer.run(sender, command_rx, cancel_token).await;
    }
    pub async fn cycle(&mut self) {
        if self.timer.get_running() {
            self.timer.stop().await;
        }
        else {
            self.timer.start().await;
        }
    }
    pub fn get_work_state(&self) -> PomodoroState {
         self.timer.get_work_state()
    }

        
    pub async fn handle_timer(time_rx: &mut tokio::sync::mpsc::Receiver<i64>, tx: tokio::sync::mpsc::Sender<Event>, cancel_token: CancellationToken) {
        loop {
            tokio::select! {
                time = time_rx.recv() => {
                    match time {
                        Some(time) => {
                            let _ = tx.send(Event::TimerTick(time)).await;
                        },
                        None => {break},
                    }
                }
                _ = cancel_token.cancelled() => {
                    break
                }

            }
        }
    }
    pub fn set_time_left(&mut self, time: i64) {
        self.timer.set_time_left(time);
        if let PomodoroState::Work(_) = self.timer.get_current_state() {
            self.timer.set_elapsed_time((self.timer.get_iteration()-1) as i64 * Timer::get_duration(&self.timer.get_work_state()) + Timer::get_duration(&self.get_work_state())-time)

        }

    }
    pub async fn set_setting(&mut self, setting: Settings) {
        self.timer.set_setting(setting).await
    }
    pub async fn handle_timer_responses(&mut self, time: i64) {
        if time == -1 && self.timer.get_iteration() < self.timer.get_total_iterations() { 
            self.timer.next_iteration().await;
        }
        else if time == -1 && self.timer.get_iteration() > self.timer.get_total_iterations() {
            self.timer.stop().await;
        }
        else if time == -1 && self.timer.get_iteration() == self.timer.get_total_iterations() {
            if let PomodoroState::Break(_) = self.timer.get_current_state() {
            self.timer.stop().await;
            }
        }
                
        else {
            self.set_time_left(time);
        }
    }
}


