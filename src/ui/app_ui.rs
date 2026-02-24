use super::{
    helpers::{HelpModal, Modal},
    pomodoro_tab::PomodoroTab,
    settings_modal::SettingsModal,
    stats_tab::StatsTab,
    ui_utils::FooterHint,
    YELLOW,
};
use crate::{app::App, ui::ui_utils::HintProvider, ui::BG, utils::tabs};
use ratatui::{
    self,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Tabs, Widget, Wrap},
    Frame,
};
pub struct AppWidget<'a> {
    app_context: &'a mut App,
}

impl Widget for &mut AppWidget<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tabs = ["Timer", "Stats"];
        let tab_titles: Vec<Span> = tabs
            .iter()
            .map(|t| Span::styled(*t, Style::default().fg(Color::White)))
            .collect();
        let selected_tab = self.app_context.selected_tab();

        let tabs_widget = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW)),
            )
            .highlight_style(Style::default().fg(Color::Rgb(240, 94, 90)))
            .select::<usize>(selected_tab.into());

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab titles
                Constraint::Min(1),    // Main content area
                Constraint::Length(1), // footer
            ])
            .split(area);

        let tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(18)])
            .split(layout[0]);

        self.app_context.set_terminal_too_small(false);
        match selected_tab {
            tabs::Tabs::TimerTab => {
                if window_too_small(20, 10, area, buf) {
                    self.app_context.set_terminal_too_small(true);
                    return;
                }
                let pomodoro_tab = PomodoroTab::new(self.app_context.pomodoro());
                let mut hints = pomodoro_tab.provide_hints();
                hints.append(&mut self.provide_hints());
                pomodoro_tab.render(layout[1], buf);
                self.render_footer(layout[2], buf, hints);
            }
            tabs::Tabs::StatsTab => {
                if window_too_small(65, 30, area, buf) {
                    self.app_context.set_terminal_too_small(true);
                    return;
                }
                let (rendered_stats, hints) = if let Some(stats_client) =
                    self.app_context.pomodoro_mut().pixela_client_as_mut()
                {
                    let mut stats = StatsTab::new(stats_client);
                    let hints = stats.provide_hints();
                    stats.render(layout[1], buf);
                    (true, hints)
                } else {
                    (false, self.provide_hints())
                };
                if !rendered_stats {
                    self.render_stats(layout[1], buf);
                }
                self.render_footer(layout[2], buf, hints);
            }
        }
        tabs_widget.render(tab_layout[0], buf);
        AppWidget::set_background(area, buf);
        if let Some(popup) = self.app_context.popup_as_mut() {
            AppWidget::dim_background(area, buf);
            popup.render(area, buf);
            return;
        }
        if let Some(modal) = self.app_context.modal() {
            AppWidget::dim_background(area, buf);
            match modal {
                Modal::HelperModal => {
                    HelpModal::new(self.app_context.selected_tab()).render(area, buf)
                }
                Modal::SettingsModal => {
                    let settings_guard = self.app_context.get_settings_ref();
                    let settings = settings_guard.borrow();
                    let timer = &self.app_context.pomodoro().timer;
                    let settings_tab = SettingsModal::new(
                        self.app_context.selected_tab(),
                        &settings,
                        timer,
                        self.app_context.pomodoro().pixela_client(),
                    );

                    settings_tab.render(area, buf);
                }
            }
        }
    }
}
fn window_too_small(
    min_width: u16,
    min_height: u16,
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
) -> bool {
    if !(area.width < min_width || area.height < min_height) {
        return false;
    }
    let warning = Paragraph::new(format!(
        "Terminal too small {}x{}, please resize to at least {}x{}",
        area.width, area.height, min_width, min_height
    ))
    .centered()
    .wrap(Wrap { trim: true });
    warning.render(area, buf);
    return true;
}
impl<'a> AppWidget<'a> {
    pub fn new(app_context: &'a mut App) -> Self {
        Self { app_context }
    }

    pub fn render_footer(&self, area: Rect, buf: &mut Buffer, hints: Vec<FooterHint>) {
        let footer_text = hints
            .iter()
            .map(|hint| format!("{}: {}", hint.key, hint.hint))
            .collect::<Vec<_>>()
            .join(" | ");

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            );

        footer.render(area, buf);
    }
    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new("Stats are turned off".to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightGreen));
        text.render(area, buf);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    fn set_background(area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).expect("Should work");
                if cell.style().bg == Some(Color::Reset) {
                    cell.set_style(cell.style().bg(BG));
                }
            }
        }
    }
    fn dim_background(area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).expect("Should work");
                cell.modifier.insert(Modifier::DIM);
            }
        }
    }
}
