use std::{fmt::format, num, process::exit, sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use std::io;
use num_traits::PrimInt;
use ratatui::{self, buffer::{self, Buffer}, layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{palette::tailwind, Color, Modifier, Style, Stylize}, symbols::{self, border}, text::{Line, Span, Text}, widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph, Tabs, Widget}, DefaultTerminal, Frame};
use crate::timer::*;
use crate::romodoro::Pomodoro;

const DEFAULT_WORK: i64 = 1800;
const DEFAULT_BREAK: i64 = 300;
const WORK_TIME_INCR: i64 = 900;
const BREAK_TIME_INCR: i64 = 60;

#[derive(Debug)]
pub struct App {
    exit: bool,
    pomodoro: Pomodoro,
    selected_tab: usize,
    settings: SettingsTab,
}
pub enum Event {
    TimerTick(i64),
    KeyPress(KeyEvent),
}
impl App {
    pub fn new(pomodoro: Pomodoro)-> Self {
        App{pomodoro, exit:false, selected_tab: 0, settings: SettingsTab{selected_setting: 0, work_time:DEFAULT_WORK, break_time:DEFAULT_BREAK}}
    }
    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,  
        mut rx: tokio::sync::mpsc::Receiver<Event>,
        tx: tokio::sync::mpsc::Sender<Event>,
        mut time_rx:tokio::sync::mpsc::Receiver<i64>,
        )
        -> io::Result<()> {

        let tx_inputs = tx.clone();
        let tx_timer = tx.clone();
        let cancelation_token = tokio_util::sync::CancellationToken::new();
        let input_cancel = cancelation_token.clone();
        let timer_comm_cancel = cancelation_token.clone();
        let timer_cancel = cancelation_token.clone();
        let input_task = tokio::spawn(async move {App::handle_inputs(tx_inputs, input_cancel).await;});
        let timer_task = tokio::spawn(async move {Pomodoro::handle_timer(&mut time_rx, tx_timer, timer_comm_cancel).await;});
        self.pomodoro.create_timer(timer_cancel);
        terminal.draw(|frame| self.draw(frame))?;
        while !self.exit {
            if let Some(event) = rx.recv().await {
                match event {
                    Event::KeyPress(key) => {self.handle_key_event(key).await;},
                    Event::TimerTick(time) => {self.pomodoro.set_time_left(time);}
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
         //global
         match key_event.code {
             KeyCode::Char('q') => self.exit(),
             KeyCode::Tab => self.change_tab(),
             _ => {},
         }
         match self.selected_tab {
             0 => {
                 // timer
                 match key_event.code {
                     KeyCode::Char(' ') => self.pomodoro.cycle().await,
                     _ => {},
                 }
             },
             1 => match key_event.code {
                 // settings
                 KeyCode::Down => self.settings.select_down(),
                 KeyCode::Up => self.settings.select_up(),
                 KeyCode::Right => self.settings.increment(),
                 KeyCode::Left => self.settings.decrement(),
                 KeyCode::Char(' ') => self.update_settings().await,
                 _ => {},
             }
             _ => {},
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
    async fn update_settings(&mut self) {
        let break_time = self.settings.get_setting(Settings::BreakTime(None));
        let work_time = self.settings.get_setting(Settings::WorkTime(None));
        let iterations = self.settings.get_setting(Settings::Iterations(None));
        let current_break_time: Settings = self.pomodoro.get_break_state().into();
        let current_work_time: Settings = self.pomodoro.get_work_state().into();
        let current_iterations: Settings = Settings::Iterations(Some(*self.pomodoro.get_iterations()));
        if current_break_time != break_time {
            self.pomodoro.send_commands(TimerCommand::Customize(break_time)).await;
            self.pomodoro.timer.set_setting(break_time);
        }
        if current_work_time != work_time {
            self.pomodoro.send_commands(TimerCommand::Customize(work_time)).await;
            self.pomodoro.timer.set_setting(work_time);
        }
        if current_iterations != iterations {
            self.pomodoro.send_commands(TimerCommand::Customize(iterations)).await;
            self.pomodoro.timer.set_setting(iterations);
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
                .border_style(Style::default().fg(Color::Rgb(215,153,33)))) // Match Pomodoro timer border color
            .highlight_style(Style::default().fg(Color::Rgb(240,94,90))) // Pomodoro color
            .select(selected_tab);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab titles
                Constraint::Min(1),   // Main content area
                Constraint::Max(1), // footer
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
            0 => self.pomodoro.render(layout[1], buf),
            1 => self.settings.render(layout[1], buf),
            2 => self.render_stats(layout[1], buf),
            _ => {}
        }
    self.render_footer(layout[2], buf);
    }
}
impl App {
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let footer_text = match self.selected_tab {
            0 => "Space: Start/Stop | Tab: Next Tab | Q: Quit",
            1 => "↑↓: Select | ←→: Adjust Value | Space: Confirm | Tab: Next Tab | Q: Quit",
            _ => "Tab: Next Tab | Q: Quit",
        };

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC));

        footer.render(area, buf);
    }
    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new("Stats Page (Placeholder)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightGreen));
        text.render(area, buf);
    }
}
#[derive(Debug, Clone)]
pub struct SettingsTab {
    selected_setting: usize,
    work_time : i64,
    break_time : i64
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
        
        
impl Widget for &SettingsTab {
        fn render(self, area: Rect, buf: &mut Buffer) {
            let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        let work_time_text = Paragraph::new("Work Time (hours)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));

        let break_time_text = Paragraph::new("Break Time (minutes)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));

            let work_time_style = if self.selected_setting == 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen)
            };

            let break_time_style = if self.selected_setting == 1 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen)
            };

            let work_time_value = Paragraph::new(format!("{:.1}", self.work_time))
                .alignment(Alignment::Center)
                .style(work_time_style);

            let break_time_value = Paragraph::new(format!("{}", self.break_time))
                .alignment(Alignment::Center)
                .style(break_time_style);

            work_time_text.render(layout[0], buf);
            work_time_value.render(layout[1], buf);
            break_time_text.render(layout[2], buf);
            break_time_value.render(layout[3], buf);
    }
}
impl SettingsTab {
    fn get_setting(&self, setting:Settings) -> Settings {
        match setting {
            Settings::BreakTime(_) => {
                Settings::BreakTime(Some(self.break_time))
            }
            Settings::WorkTime(_) => {
                Settings::WorkTime(Some(self.work_time))
            }
            Settings::Iterations(_) => {
                Settings::Iterations(Some(4))
            }

        }
    }

    fn select_down(&mut self) {
        if self.selected_setting == 1 {
            self.selected_setting = 0;
        } else { self.selected_setting += 1}
    }
    fn select_up(&mut self) {
        if self.selected_setting == 0 {
            self.selected_setting = 1;
        } else { self.selected_setting -= 1}
    }
    fn decrement(&mut self) {
        match self.selected_setting {
            0 if self.work_time - WORK_TIME_INCR != 0 => {self.work_time -= WORK_TIME_INCR},
            1 if self.break_time - BREAK_TIME_INCR != 0  => {self.break_time -= BREAK_TIME_INCR},
            _ => {},
        }
    }
    fn increment(&mut self) {
        match self.selected_setting {
            0 => {self.work_time += WORK_TIME_INCR},
            1 => {self.break_time += BREAK_TIME_INCR},
            _ => {},
        }
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
