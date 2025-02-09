
use ratatui::{self, buffer::{self, Buffer}, layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{palette::tailwind, Color, Modifier, Style, Stylize}, symbols::{self, border}, text::{Line, Span, Text}, widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph, Tabs, Widget}, DefaultTerminal, Frame};
use tokio_util::sync::CancellationToken;
use crate::{app::Event, timer::*};

#[derive(Debug)]
pub struct Pomodoro {
    pub timer : Timer,
    time_sender: tokio::sync::mpsc::Sender<i64>,
    command_tx: tokio::sync::mpsc::Sender<TimerCommand>,
    command_rx: Option<tokio::sync::mpsc::Receiver<TimerCommand>>,
    work_state: PomodoroState, break_state: PomodoroState, iterations: u8,
}
impl Pomodoro {
    pub fn new(work_state: PomodoroState, break_state: PomodoroState, iterations: u8,time_sender: tokio::sync::mpsc::Sender<i64>, command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,command_tx: tokio::sync::mpsc::Sender<TimerCommand>) -> Self {
        let timer = Timer::new(work_state.clone(), break_state.clone(), iterations);
        Pomodoro {timer,work_state, break_state, iterations, command_rx: Some(command_rx), command_tx, time_sender}
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
    pub fn get_work_state(&self) -> &PomodoroState {
         &self.work_state
    }
    pub fn get_break_state(&self) -> &PomodoroState {
         &self.break_state
    }
    pub fn get_iterations(&self) -> &u8 {
         &self.iterations
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
    pub fn set_time_left(&mut self, time: i64) {
        self.timer.set_time_left(time);
    }
}
impl Widget for &Pomodoro {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
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
                Constraint::Max(8),
                Constraint::Min(10), // Bigger ASCII tomato art
            ])
            .margin(2) // Adds even more padding on the sides for a centered look
            .split(area);

        let gauge_layout = Layout::default().direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25)
            ]).split(layout[4]);
        buf.set_style(area, Style::default().fg(Color::DarkGray));
        outer_block.render(area, buf);

        now_paragraph.render(layout[0], buf);
        timer_text.render(layout[2], buf);
        count_paragraph.render(layout[3], buf);
        gauge.render(gauge_layout[1], buf);
        romodoro.render(layout[6], buf);
    }
    }
    


