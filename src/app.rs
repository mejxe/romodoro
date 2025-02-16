use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use tokio_util::sync::CancellationToken;
use ratatui::DefaultTerminal;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use crate::romodoro::Pomodoro;
use crate::settings::*;


#[derive(Debug)]
pub struct App {
    exit: bool,
    pomodoro: Pomodoro,
    selected_tab: usize,
    settings: Rc<RefCell<SettingsTab>>,
}
pub enum Event {
    TimerTick(i64),
    KeyPress(KeyEvent),
}
impl App {
    pub fn new(pomodoro: Pomodoro, settings: Rc<RefCell<SettingsTab>>)-> Self {
        App{pomodoro, exit:false, selected_tab: 0, settings}
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
        self.pomodoro.create_countdown(timer_cancel).await;
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
             KeyCode::Char('Q') => self.exit(),
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
                 KeyCode::Down => self.settings.borrow_mut().select_down(),
                 KeyCode::Up => self.settings.borrow_mut().select_up(),
                 KeyCode::Right => self.settings.borrow_mut().increment(),
                 KeyCode::Left => self.settings.borrow_mut().decrement(),
                 KeyCode::Char(' ') => self.update_settings().await,
                 KeyCode::Char('r') => self.settings.borrow_mut().restore_defaults(),
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
        let break_time = self.settings.borrow().get_pomodoro_setting(PomodoroSettings::BreakTime(None));
        let work_time = self.settings.borrow().get_pomodoro_setting(PomodoroSettings::WorkTime(None));
        let iterations = self.settings.borrow().get_pomodoro_setting(PomodoroSettings::Iterations(None));
        let current_break_time: PomodoroSettings = self.pomodoro.timer.get_break_state().into();
        let current_work_time: PomodoroSettings = self.pomodoro.timer.get_work_state().into();
        let current_iterations: PomodoroSettings = PomodoroSettings::Iterations(Some(self.pomodoro.timer.get_total_iterations()));
        if current_break_time != break_time {
            self.pomodoro.set_setting(break_time).await;
        }
        if current_work_time != work_time {
            self.pomodoro.set_setting(work_time).await;
        }
        if current_iterations != iterations {
            self.pomodoro.set_setting(iterations).await;
        }
        
    }
    pub fn get_selected_tab(&self) -> usize {
        self.selected_tab
    }
    pub fn get_settings_ref(&self) -> Rc<RefCell<SettingsTab>> {
        self.settings.clone()
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
