use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

impl App {
    pub async fn handle_help(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.set_modal(None),
            _ => {}
        };
    }
}
