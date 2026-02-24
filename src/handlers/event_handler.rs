use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::App,
    error::Error,
    popup::Popup,
    settings::Mode,
    stats::pixela::{graph::Graph, subjects::Subject},
    ui::popup::list_height,
    utils::tabs::Tabs,
};
pub enum Event {
    TimerTick(i64),
    KeyPress(KeyEvent),
    TerminalEvent,
    OverwriteTimerSettings,
    OverwriteTimerForSubject(usize),
    SendPixels,
    DeletePixel,
    RequestGraph,
    RestartTimer,
    GraphReceived(Result<Graph, Error>),
    LoggedIn(Result<Vec<Subject>, Error>),
}

impl App {
    pub async fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyPress(key) => {
                self.handle_key_event(key).await;
            }
            Event::TimerTick(time) => {
                if let Err(e) = self.pomodoro_mut().handle_timer_tick(time).await {
                    self.set_popup(e.into());
                };
            }
            Event::TerminalEvent => {}
            Event::OverwriteTimerSettings => self.overwrite_timer_settings().await,
            Event::OverwriteTimerForSubject(index) => self.overwrite_timer_for_subject(index).await,
            Event::SendPixels => {
                if let Some(pixela) = self.pomodoro_mut().pixela_client_as_mut() {
                    match pixela.send_pixels().await {
                        Ok(pixels) => self.set_popup(Popup::pixel_list(
                            "Sucessfuly sent these pixels".into(),
                            pixels,
                        )),
                        Err(e) => self.set_popup(e.into()),
                    }
                }
            }
            Event::DeletePixel => {
                if let Some(pixela_client) = self.pomodoro_mut().pixela_client_as_mut() {
                    if let Err(e) = pixela_client.delete_pixel() {
                        self.set_popup(e.into());
                    }
                }
            }
            Event::RequestGraph => {
                let tx_clone = self.event_tx().clone();
                if let Some(client) = self.pomodoro_mut().pixela_client_as_mut() {
                    if let Some(index) = client.subjects.state().selected() {
                        let user = client.user.clone();
                        let rq_client = client.client.clone();
                        let subject = client.get_subject(index).unwrap();

                        tokio::spawn(async move {
                            let result = Graph::download_graph(user, rq_client, subject).await;

                            if let Err(e) = tx_clone.send(Event::GraphReceived(result)).await {
                                eprintln!("Failed to send GraphDataReceived event: {}", e);
                            }
                        });
                    }
                }
            }
            Event::GraphReceived(res) => {
                if let Some(client) = self.pomodoro_mut().pixela_client_as_mut() {
                    match res {
                        Ok(g) => client.set_current_graph(Some(g)),
                        Err(e) => self.set_popup(e.into()),
                    }
                }
            }
            Event::RestartTimer => {
                self.pomodoro_mut().timer.restart().await;
            }
            Event::LoggedIn(response) => match response {
                Ok(mut subjects) => {
                    if let Some(pixela_client) = self.pomodoro_mut().pixela_client_as_mut() {
                        pixela_client.subjects.items_mut().append(&mut subjects);
                        pixela_client.logged_in = true;
                    }
                }
                Err(err) => self.set_popup(err.into()),
            },
        }
    }
    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        //global
        match key_event.code {
            KeyCode::Char('Q') => self.exit(),
            KeyCode::Tab
                if self.popup().is_none()
                    && self.modal().is_none()
                    && !(self.settings().borrow().mode() == Mode::Input) =>
            {
                self.settings().borrow_mut().change_mode(Mode::Normal);
                self.selected_tab_mut().next();
            }
            KeyCode::Char('S') => {
                self.set_modal(Some(crate::ui::helpers::Modal::SettingsModal));
                self.set_modal_tab();
            }
            KeyCode::Char('H') => self.set_modal(Some(crate::ui::helpers::Modal::HelperModal)),
            _ => {}
        }
        if self.terminal_too_small() {
            return;
        }
        let list_height = list_height(&self.popup_size());
        if let Some(popup) = self.take_popup() {
            self.handle_popups(key_event, popup, list_height).await;
            return;
        };
        let settings_mode = self.settings().borrow().mode();
        if let Some(modal) = self.modal() {
            match modal {
                crate::ui::helpers::Modal::SettingsModal => match settings_mode {
                    Mode::Normal => self.handle_settings_normal(key_event).await,
                    Mode::Input => self.handle_settings_input(key_event).await,
                },
                crate::ui::helpers::Modal::HelperModal => self.handle_help(key_event).await,
            }
            return;
        }
        match self.selected_tab() {
            Tabs::TimerTab => self.handle_timer_tab(key_event).await,
            Tabs::StatsTab => self.handle_pixela_keybinds(key_event).await,
        }
    }
    async fn overwrite_timer_for_subject(&mut self, index: usize) {
        self.pomodoro_mut().restart_timer().await;
        self.pomodoro_mut().set_current_subject_index(index);
    }
    fn set_modal_tab(&mut self) {
        match self.selected_tab() {
            Tabs::TimerTab => self
                .settings()
                .borrow_mut()
                .set_selected_tab(crate::utils::settings_helper_structs::SettingsTabs::Pomodoro),
            Tabs::StatsTab => self
                .settings()
                .borrow_mut()
                .set_selected_tab(crate::utils::settings_helper_structs::SettingsTabs::Stats),
        }
    }
}
