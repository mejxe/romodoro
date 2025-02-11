use tokio_util::sync::CancellationToken;

use crate::{app::Event, timer::*};

#[derive(Debug)]
pub struct Pomodoro {
    pub timer : Timer,
    time_sender: tokio::sync::mpsc::Sender<i64>,
    command_tx: tokio::sync::mpsc::Sender<TimerCommand>,
    command_rx: Option<tokio::sync::mpsc::Receiver<TimerCommand>>,
}
impl Pomodoro {
    pub fn new(time_sender: tokio::sync::mpsc::Sender<i64>, command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,command_tx: tokio::sync::mpsc::Sender<TimerCommand>) -> Self {
        let timer = Timer::default();
        Pomodoro {timer, command_rx: Some(command_rx), command_tx, time_sender}
    }
    pub fn create_timer(&mut self, cancel_token: CancellationToken) { 
        let sender = self.time_sender.clone();
        let mut timer = self.timer.clone();
        let command_rx = self.command_rx.take().expect("Timer initialized");

        tokio::task::spawn({
            async move {
                timer.run(sender, command_rx, cancel_token).await;
            }
        });
    }
    pub async fn send_commands(&self, command: TimerCommand) {
        let _ = self.command_tx.send(command).await;
    }
    pub async fn cycle(&mut self) {
        if self.timer.get_running() {
            self.timer.set_running(false);
            self.send_commands(TimerCommand::Stop).await;
        }
        else {
            self.timer.set_running(true);
            self.send_commands(TimerCommand::Start).await;
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
    pub async fn handle_timer_responses(&mut self, time: i64) {
        if time == -1 && self.timer.get_iteration() < self.timer.get_total_iterations() { 
            self.send_commands(TimerCommand::Stop).await;
            self.send_commands(TimerCommand::NextIteration).await;
            self.send_commands(TimerCommand::Start).await;
            self.timer.set_running(false);
            self.timer.next_iteration();
            self.timer.set_running(true);
        }
        else if time == -1 && self.timer.get_iteration() > self.timer.get_total_iterations() {
            self.timer.set_running(false);
            self.send_commands(TimerCommand::Stop).await;
        }
        else if time == -1 && self.timer.get_iteration() == self.timer.get_total_iterations() {
            if let PomodoroState::Break(_) = self.timer.get_current_state() {
                self.timer.set_running(false);
                self.send_commands(TimerCommand::Stop).await;
            }
        }
                
        else {
            self.set_time_left(time);
        }
    }
}


