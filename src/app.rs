
use std::{fmt::format, process::exit, sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use std::io;
use ratatui::{self, buffer::{self, Buffer}, layout::{Alignment, Constraint, Layout, Rect}, style::{palette::tailwind, Color, Stylize}, symbols::{self, border}, text::{Line, Text}, widgets::{Block, BorderType, Borders, Padding, Paragraph, Tabs, Widget}, DefaultTerminal, Frame};

use crate::timer::*;


#[derive(Debug)]
pub struct App {
    exit: bool,
    timer: Timer,
    time_sender: tokio::sync::mpsc::Sender<i64>,
    command_tx: tokio::sync::mpsc::Sender<TimerCommand>,
}
pub enum Event {
    TimerTick(i64),
    KeyPress(KeyEvent),
}
impl App {
    pub fn new(timer: Timer, time_sender: tokio::sync::mpsc::Sender<i64>,command_tx: tokio::sync::mpsc::Sender<TimerCommand> )-> Self {
        App{timer, exit:false, time_sender, command_tx}
    }
    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,  
        mut rx: tokio::sync::mpsc::Receiver<Event>,
        tx: tokio::sync::mpsc::Sender<Event>,
        mut time_rx:tokio::sync::mpsc::Receiver<i64>,
        mut command_rx: tokio::sync::mpsc::Receiver<TimerCommand>
        )
        -> io::Result<()> {

        let tx_inputs = tx.clone();
        let tx_timer = tx.clone();
        let cancelation_token = tokio_util::sync::CancellationToken::new();
        let input_cancel = cancelation_token.clone();
        let timer_comm_cancel = cancelation_token.clone();
        let timer_cancel = cancelation_token.clone();
        let input_task = tokio::spawn(async move {App::handle_inputs(tx_inputs, input_cancel).await;});
        let timer_task = tokio::spawn(async move {App::handle_timer(&mut time_rx, tx_timer, timer_comm_cancel).await;});
        self.create_timer(command_rx, timer_cancel);
        terminal.draw(|frame| self.draw(frame))?;
        while !self.exit {
            if let Some(event) = rx.recv().await {
                match event {
                    Event::KeyPress(key) => {self.handle_key_event(key).await;},
                    Event::TimerTick(time) => {self.timer.set_time_left(time);}
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        cancelation_token.cancel();
        timer_task.await.unwrap();
        input_task.await.unwrap();
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
     async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => self.start_timer().await,
            KeyCode::Char('s') => self.stop_timer().await,
            _ => {},
        }
    }

     pub fn create_timer(&mut self, command_rx: tokio::sync::mpsc::Receiver<TimerCommand>, cancel_token: CancellationToken) { 
         let sender = self.time_sender.clone();
         let mut timer = self.timer.clone();

         tokio::task::spawn({
             async move {
                 timer.run(sender, command_rx, cancel_token).await;
             }
         });
        
    }
    async fn send_commands(&self, command: TimerCommand) {
        let _ = self.command_tx.send(command).await;
    }

        
    pub async fn handle_timer(time_rx: &mut tokio::sync::mpsc::Receiver<i64>, tx: tokio::sync::mpsc::Sender<Event>, cancel_token: CancellationToken) {
        loop {
            tokio::select! {
                time = time_rx.recv() => {
                    match time {
                        Some(time) => {let _ = tx.send(Event::TimerTick(time)).await;},
                        None => {break},
                    }
                }
                _ = cancel_token.cancelled() => {
                    break
                }

            }
        }
    }

    async fn handle_inputs(tx: tokio::sync::mpsc::Sender<Event>, cancel_token: CancellationToken ) -> std::io::Result<()>{
        loop {
            tokio::select! {
                result = tokio::task::spawn_blocking(|| event::read()) => {
            match result.unwrap().unwrap() {
                crossterm::event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let _ = tx.send(Event::KeyPress(key_event)).await;
                    println!("sent");
                }
                _ => {}
            };
                }
                _ = cancel_token.cancelled() => {

                    break
                }
            }
        }
        Ok(())
    }

     async fn stop_timer(&mut self) {
         self.send_commands(TimerCommand::Stop).await;
    }
     async fn start_timer(&mut self) {
         self.send_commands(TimerCommand::Start).await;
     }


    fn exit(&mut self) {
        self.exit = true;
    }
}


impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let time = self.timer.get_timeleft();
        let block = Block::default()
            .title(" Timer ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let timer_text = Paragraph::new(format!("{time}"))
            .block(block).
            alignment(Alignment::Center);
        timer_text.render(area, buf);

    }
}

#[cfg(test)] 
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn multithread_works() {
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let mut app = App::new(Timer::new(PomodoroState::Work(2), PomodoroState::Break(1), 2), rx, tx);
        app.start_timer();

    }
}
