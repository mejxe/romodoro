use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use tokio_util::sync::CancellationToken;
use ratatui::DefaultTerminal;
use std::io;
use crate::timer::*;
use crate::romodoro::Pomodoro;
use crate::settings::*;
use crate::{DEFAULT_WORK, DEFAULT_BREAK, DEFAULT_ITERATIONS};


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
        App{pomodoro, exit:false, selected_tab: 0, settings: SettingsTab::default()}
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
                    Event::TimerTick(time) => {self.pomodoro.handle_timer_responses(time).await;}
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        cancelation_token.cancel();
        timer_task.await.unwrap();
        input_task.abort();
        Ok(())
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
        let current_break_time: Settings = self.pomodoro.timer.get_break_state().into();
        let current_work_time: Settings = self.pomodoro.timer.get_work_state().into();
        let current_iterations: Settings = Settings::Iterations(Some(self.pomodoro.timer.get_total_iterations()));
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
    pub fn get_selected_tab(&self) -> usize {
        self.selected_tab
    }
    pub fn get_settings_ref(&self) -> &SettingsTab {
        &self.settings
    }
    pub fn get_pomodoro_ref(&self) -> &Pomodoro {
        &self.pomodoro
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
