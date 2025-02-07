use std::io;
use std::process::exit;
use std::sync::Arc;
use std::sync::Mutex;
use pomodoro::app::*;
use pomodoro::timer::*;

#[tokio::main]
async fn main() -> std::io::Result<()>{
    let mut input = String::new();
    println!("How long to study?");
    let _ = io::stdin().read_line(&mut input);
    let work_state = PomodoroState::Work(input.trim().parse::<i64>().unwrap());
    input = "".to_string();
    println!("How long a break?");
    let _ = io::stdin().read_line(&mut input);
    println!("{input}");
    let break_state = PomodoroState::Break(input.trim().parse::<i64>().unwrap());
    let (tx, rx) = tokio::sync::mpsc::channel(4);
    let (tx_events,  rx_events) = tokio::sync::mpsc::channel(32);
    let (tx_commands, rx_commands) = tokio::sync::mpsc::channel(4);
    let timerr = Timer::new(work_state, break_state, 4);
    let mut terminal = ratatui::init();
    let mut app = App::new(timerr, tx, tx_commands);
    let app_result = app.run(&mut terminal,rx_events,tx_events,rx,rx_commands).await;
    ratatui::restore();
    app_result
//    while iterations <= 4 {
//        let mut duration: u8 = match current_state {
//            PomodoroState::Work(_) => {
//                current_state = &break_state;
//                println!("Work time!");
//                iterations += 1;
//                work_state.get_duration()
//            },
//            PomodoroState::Break(_) => {
//                current_state = &work_state;
//                println!("Break time!");
//                break_state.get_duration()
//            },
//        };
//        while duration > 0 {
//            println!("{duration} seconds left...");
//            duration -= 1;
//            sleep(time::Duration::from_secs(1));
//        }
//    }
//    println!("You have worked for {} minutes.", iterations*work_state.get_duration())
}

