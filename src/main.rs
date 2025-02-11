use crossterm::terminal;
use pomodoro::app::*;
use pomodoro::romodoro::*;
// ALPHA 0.1

#[tokio::main]
async fn main() -> std::io::Result<()>{
    let (tx, rx) = tokio::sync::mpsc::channel(4);
    let (tx_events,  rx_events) = tokio::sync::mpsc::channel(32);
    let (tx_commands, rx_commands) = tokio::sync::mpsc::channel(4);
    let pomodoro = Pomodoro::new(tx, rx_commands, tx_commands);

    terminal::enable_raw_mode()?;
    let mut terminal = ratatui::init();
    let mut app = App::new(pomodoro);
    let app_result = app.run(&mut terminal,rx_events,tx_events,rx).await; // mainloop
    terminal::disable_raw_mode()?;

    ratatui::restore();
    app_result
}
