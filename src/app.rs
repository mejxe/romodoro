
use std::{fmt::format, process::exit, sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use std::io;
use ratatui::{self, buffer::{self, Buffer}, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{palette::tailwind, Color, Modifier, Style, Stylize}, symbols::{self, border}, text::{Line, Span, Text}, widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph, Tabs, Widget}, DefaultTerminal, Frame};

use crate::timer::*;


#[derive(Debug)]
pub struct App {
    exit: bool,
    timer: Timer,
    selected_tab: usize,
    time_sender: tokio::sync::mpsc::Sender<i64>,
    command_tx: tokio::sync::mpsc::Sender<TimerCommand>,
}
pub enum Event {
    TimerTick(i64),
    KeyPress(KeyEvent),
}
impl App {
    pub fn new(timer: Timer, time_sender: tokio::sync::mpsc::Sender<i64>,command_tx: tokio::sync::mpsc::Sender<TimerCommand> )-> Self {
        App{timer, exit:false, time_sender, command_tx, selected_tab: 0}
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
        input_task.abort();
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
     async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => self.send_commands(TimerCommand::Start).await,
            KeyCode::Char('s') => self.send_commands(TimerCommand::Stop).await,
            KeyCode::Tab => self.change_tab(),
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
            match event::read()? {
                crossterm::event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let _ = tx.send(Event::KeyPress(key_event)).await;
                    if cancel_token.is_cancelled() {return Ok(())}
                },
                _ => { if cancel_token.is_cancelled() {return Ok(())} }
            };

        }
    }

     fn change_tab(&mut self) {
         if self.selected_tab == 2 {
             self.selected_tab = 0;
         } else { self.selected_tab += 1; }
     }


    fn exit(&mut self) {
        self.exit = true;
    }
}
// UI 
impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tabs = ["Pomodoro Timer", "Settings", "Stats"];
        let tab_titles: Vec<Span> = tabs.iter().map(|t| Span::styled(*t, Style::default().fg(Color::White))).collect();
        let selected_tab = self.selected_tab; // Assume this is tracked in the App struct

        let tabs_widget = Tabs::new(tab_titles)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title(" Menu ")
                .border_style(Style::default().fg(Color::Rgb(99, 150, 99)))) // Match Pomodoro timer border color
            .highlight_style(Style::default().fg(Color::Rgb(240,94,90))) // Pomodoro color
            .select(selected_tab);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab titles
                Constraint::Min(1),   // Main content area
            ])
            .split(area);

        let tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Max(38), // Limit width of tabs
                Constraint::Min(1),
            ])
            .split(layout[0]);

        tabs_widget.render(tab_layout[0], buf);

        match selected_tab {
            0 => self.render_pomodoro(layout[1], buf),
            1 => self.render_settings(layout[1], buf),
            2 => self.render_stats(layout[1], buf),
            _ => {}
        }
    }
}
impl App {
    fn render_pomodoro(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let time = self.timer.get_timeleft();
        let total_time = self.timer.get_total_time(); // Placeholder for total time
        let work_period_time = self.timer.get_total_time()/self.timer.get_total_iterations() as i64;
        let elapsed_time = work_period_time * (self.timer.get_iteration() as i64 -1) + (work_period_time - time);
        let now_text = format!("Now: {}", self.timer.get_work_state());
        let progress = (elapsed_time) as f64 / total_time as f64;
        let iterations_text = format!("{}/{} iterations", self.timer.get_iteration(), self.timer.get_total_iterations());

        let outer_block = Block::default()
            .title(" Pomodoro Timer ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::Rgb(215,153,33))); // Gruvbox dark border

        let timer_text = Paragraph::new(format!("Time Left: {time} seconds"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::NONE));

        let now_paragraph = Paragraph::new(now_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::DIM));

        let count_paragraph = Paragraph::new(iterations_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(69,133,136)).add_modifier(Modifier::ITALIC));

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).border_style(Color::Rgb(99, 150, 99)).border_type(BorderType::Thick))
            .gauge_style(Style::default().fg(Color::Rgb(85, 158, 85)))
            .ratio(progress);

        let romodoro_ascii = r"
          _____                   _______                   _____                   _______                   _____                   _______                   _____                   _______         
         /\    \                 /::\    \                 /\    \                 /::\    \                 /\    \                 /::\    \                 /\    \                 /::\    \        
        /::\    \               /::::\    \               /::\____\               /::::\    \               /::\    \               /::::\    \               /::\    \               /::::\    \       
       /::::\    \             /::::::\    \             /::::|   |              /::::::\    \             /::::\    \             /::::::\    \             /::::\    \             /::::::\    \      
      /::::::\    \           /::::::::\    \           /:::::|   |             /::::::::\    \           /::::::\    \           /::::::::\    \           /::::::\    \           /::::::::\    \     
     /:::/\:::\    \         /:::/~~\:::\    \         /::::::|   |            /:::/~~\:::\    \         /:::/\:::\    \         /:::/~~\:::\    \         /:::/\:::\    \         /:::/~~\:::\    \    
    /:::/__\:::\    \       /:::/    \:::\    \       /:::/|::|   |           /:::/    \:::\    \       /:::/  \:::\    \       /:::/    \:::\    \       /:::/__\:::\    \       /:::/    \:::\    \   
   /::::\   \:::\    \     /:::/    / \:::\    \     /:::/ |::|   |          /:::/    / \:::\    \     /:::/    \:::\    \     /:::/    / \:::\    \     /::::\   \:::\    \     /:::/    / \:::\    \  
  /::::::\   \:::\    \   /:::/____/   \:::\____\   /:::/  |::|___|______   /:::/____/   \:::\____\   /:::/    / \:::\    \   /:::/____/   \:::\____\   /::::::\   \:::\    \   /:::/____/   \:::\____\ 
 /:::/\:::\   \:::\____\ |:::|    |     |:::|    | /:::/   |::::::::\    \ |:::|    |     |:::|    | /:::/    /   \:::\ ___\ |:::|    |     |:::|    | /:::/\:::\   \:::\____\ |:::|    |     |:::|    |
/:::/  \:::\   \:::|    ||:::|____|     |:::|    |/:::/    |:::::::::\____\|:::|____|     |:::|    |/:::/____/     \:::|    ||:::|____|     |:::|    |/:::/  \:::\   \:::|    ||:::|____|     |:::|    |
\::/   |::::\  /:::|____| \:::\    \   /:::/    / \::/    / ~~~~~/:::/    / \:::\    \   /:::/    / \:::\    \     /:::|____| \:::\    \   /:::/    / \::/   |::::\  /:::|____| \:::\    \   /:::/    / 
 \/____|:::::\/:::/    /   \:::\    \ /:::/    /   \/____/      /:::/    /   \:::\    \ /:::/    /   \:::\    \   /:::/    /   \:::\    \ /:::/    /   \/____|:::::\/:::/    /   \:::\    \ /:::/    /  
       |:::::::::/    /     \:::\    /:::/    /                /:::/    /     \:::\    /:::/    /     \:::\    \ /:::/    /     \:::\    /:::/    /          |:::::::::/    /     \:::\    /:::/    /   
       |::|\::::/    /       \:::\__/:::/    /                /:::/    /       \:::\__/:::/    /       \:::\    /:::/    /       \:::\__/:::/    /           |::|\::::/    /       \:::\__/:::/    /    
       |::| \::/____/         \::::::::/    /                /:::/    /         \::::::::/    /         \:::\  /:::/    /         \::::::::/    /            |::| \::/____/         \::::::::/    /     
       |::|  ~|                \::::::/    /                /:::/    /           \::::::/    /           \:::\/:::/    /           \::::::/    /             |::|  ~|                \::::::/    /      
       |::|   |                 \::::/    /                /:::/    /             \::::/    /             \::::::/    /             \::::/    /              |::|   |                 \::::/    /       
       \::|   |                  \::/____/                /:::/    /               \::/____/               \::::/    /               \::/____/               \::|   |                  \::/____/        
        \:|   |                   ~~                      \::/    /                 ~~                      \::/____/                 ~~                      \:|   |                   ~~              
         \|___|                                            \/____/                                           ~~                                                \|___|                                   
        ";
        
        let romodoro = Paragraph::new(romodoro_ascii)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(240,94,90)));
        
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Smaller "Now: {}"
                Constraint::Length(2),
                Constraint::Length(4), // Bigger timer text
                Constraint::Length(2), // Smaller "{}/{} iterations"
                Constraint::Length(3), // Progress bar
                Constraint::Max(5),
                Constraint::Min(10), // Bigger ASCII tomato art
            ])
            .margin(5) // Adds even more padding on the sides for a centered look
            .split(area);

        let inner_area = outer_block.inner(area);
        buf.set_style(area, Style::default().fg(Color::DarkGray));
        outer_block.render(area, buf);

        now_paragraph.render(layout[0], buf);
        timer_text.render(layout[2], buf);
        count_paragraph.render(layout[3], buf);
        gauge.render(layout[4], buf);
        romodoro.render(layout[6], buf);
    }
        fn render_settings(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new("Settings Page (Placeholder)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightCyan));
        text.render(area, buf);
    }

    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new("Stats Page (Placeholder)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightGreen));
        text.render(area, buf);
    }
}

#[cfg(test)] 
mod test {
    use super::*;

  //  #[test]
 //   #[ignore]
 //   fn multithread_works() {
 //       let (tx, rx) = tokio::sync::mpsc::channel(4);
 //       let mut app = App::new(Timer::new(PomodoroState::Work(2), PomodoroState::Break(1), 2), rx, tx);
 //       app.start_timer();

 //   }
}//
