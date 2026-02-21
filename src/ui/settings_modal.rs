use crate::{
    settings::{Mode, Settings},
    stats::pixela::pixela_client::PixelaClient,
    timers::{counters::CounterMode, timer::Timer},
    ui::{popup::popup_area, RED, YELLOW},
    utils::tabs::Tabs,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Styled, Stylize},
    text::{Line, Text},
    widgets::{Block, Clear, Padding, Paragraph, Widget, Wrap},
};

use super::BLUE;
pub struct SettingsModal<'a> {
    current_tab: Tabs,
    settings: &'a Settings,
    timer: &'a Timer,
    stats: Option<&'a PixelaClient>,
}

impl<'a> SettingsModal<'a> {
    pub fn new(
        current_tab: Tabs,
        settings: &'a Settings,
        timer: &'a Timer,
        stats: Option<&'a PixelaClient>,
    ) -> Self {
        Self {
            current_tab,
            settings,
            timer,
            stats,
        }
    }
}
impl Widget for &SettingsModal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.current_tab {
            Tabs::TimerTab => {
                let modal_area = popup_area(area, 30, 40);
                Clear.render(modal_area, buf);
                SettingsModal::render_timer_settings(modal_area, buf, self.settings, self.timer);
            }
            Tabs::StatsTab => {
                let modal_area = popup_area(area, 40, 20);
                Clear.render(modal_area, buf);
                SettingsModal::render_stats_settings(modal_area, buf, self.settings)
            }
        }
    }
}
impl SettingsModal<'_> {
    fn render_timer_settings(area: Rect, buf: &mut Buffer, settings: &Settings, timer: &Timer) {
        let selected_setting = settings.selected_setting;
        let mode = settings.mode();
        let block = Block::bordered()
            .title("Settings")
            .padding(Padding::symmetric(2, 1))
            .border_style(Style::new().fg(YELLOW));
        let mut lines = Vec::new();

        // --- Mode ---
        lines.push(Line::raw("Mode").centered().bold().fg(Color::Green));
        let mode_str = match settings.counter_mode() {
            CounterMode::Countup => "Flowmodoro",
            CounterMode::Countdown => "Pomodoro",
        };
        lines.push(
            Line::raw(format!("Current: {}", mode_str))
                .centered()
                .style(SettingsModal::highlight_selected(selected_setting, 0, mode)),
        );
        lines.push(Line::raw(""));

        // --- Pomodoro ---
        lines.push(Line::raw("Pomodoro").centered().bold().fg(Color::Blue));
        if settings.counter_mode() == CounterMode::Countup {
            lines.push(
                Line::raw("Pomodoro settings not available")
                    .centered()
                    .fg(Color::Gray),
            );
        } else {
            lines.push(
                Line::raw(format!(
                    "Work time: {} min",
                    settings.timer_settings.work_time / 60
                ))
                .centered()
                .style(SettingsModal::highlight_selected(
                    selected_setting,
                    1,
                    mode,
                )),
            );
            lines.push(
                Line::raw(format!(
                    "Break time: {} min",
                    settings.timer_settings.break_time / 60
                ))
                .centered()
                .style(SettingsModal::highlight_selected(
                    selected_setting,
                    2,
                    mode,
                )),
            );
            lines.push(
                Line::raw(format!(
                    "Iterations: {} iter(s)",
                    settings.timer_settings.iterations
                ))
                .centered()
                .style(SettingsModal::highlight_selected(
                    selected_setting,
                    3,
                    mode,
                )),
            );
        }
        lines.push(Line::raw(""));

        // --- Preferences ---
        lines.push(Line::raw("Preferences").centered().bold().fg(Color::Red));
        lines.push(
            Line::raw(format!(
                "Pause after every cycle: {}",
                if settings.ui_settings.pause_after_state_change {
                    "yes"
                } else {
                    "no"
                }
            ))
            .centered()
            .style(SettingsModal::highlight_selected(selected_setting, 4, mode)),
        );
        lines.push(
            Line::raw(format!(
                "Make the numbers smaller: {}",
                if settings.ui_settings.hide_work_countdown {
                    "yes"
                } else {
                    "no"
                }
            ))
            .centered()
            .style(SettingsModal::highlight_selected(selected_setting, 5, mode)),
        );
        lines.push(Line::raw(""));
        if !settings.do_pomodoro_settings_match(timer) {
            lines.push(
                Line::raw("Space to apply settings")
                    .centered()
                    .bold()
                    .fg(RED),
            )
        }
        let paragraph = Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: true })
            .block(block);
        paragraph.render(area, buf);
    }
    fn render_stats_settings(area: Rect, buf: &mut Buffer, settings: &Settings) {
        let block = Block::bordered()
            .title("Settings")
            .padding(Padding::symmetric(2, 1))
            .border_style(Style::new().fg(YELLOW));
        block.render(area, buf);
        let selected_setting = settings.selected_setting;
        let mode = settings.mode();
        // --- Stats ---
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Length(1), // Stats Enable
                Constraint::Length(1), // Username
                Constraint::Length(1), // API Key
            ])
            .split(area);

        // --- Header ---
        Paragraph::new(Line::raw("Login").centered().bold().fg(Color::Yellow)).render(rows[0], buf);

        let split_row = |row_area: Rect| -> [Rect; 2] {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(row_area);
            [cols[0], cols[1]]
        };

        // --- Enable stats ---
        let stats_cols = split_row(rows[1]);
        let stats_val = if settings.stats_setting.stats_on {
            "yes"
        } else {
            "no"
        };
        Paragraph::new("Enable stats: ")
            .right_aligned()
            .render(stats_cols[0], buf);
        Paragraph::new(stats_val)
            .left_aligned()
            .style(SettingsModal::highlight_selected(selected_setting, 0, mode))
            .render(stats_cols[1], buf);

        // --- Username ---
        let max_text_len = (area.width / 2) as usize;
        let user_cols = split_row(rows[2]);
        let username = {
            let u = settings
                .stats_setting
                .pixela_username
                .as_deref()
                .unwrap_or(" ");
            if u.is_empty() {
                if Mode::Input == mode && selected_setting == 1 {
                    "█".to_string()
                } else {
                    "-".to_string()
                }
            } else {
                let mut disp = u.to_string();
                if Mode::Input == mode && selected_setting == 1 {
                    disp.push('█');
                }
                if disp.len() >= max_text_len {
                    let difference = disp.len() - max_text_len;
                    disp = disp.split_off(difference);
                }
                disp
            }
        };
        Paragraph::new("Username: ")
            .right_aligned()
            .render(user_cols[0], buf);
        Paragraph::new(Text::styled(
            username,
            SettingsModal::highlight_selected(selected_setting, 1, mode),
        ))
        .left_aligned()
        .render(user_cols[1], buf);

        // --- Api key ---
        let api_cols = split_row(rows[3]);
        let token_display = match (mode, selected_setting) {
            (Mode::Input, 2) => settings
                .stats_setting
                .pixela_token
                .as_ref()
                .map(|t| {
                    let mut display_string =
                        "•".repeat(t.len().min(max_text_len - 2).saturating_sub(1));
                    display_string.push(t.chars().last().unwrap_or('\0'));
                    display_string.push('█');
                    display_string
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "-".to_string()),
            _ => settings
                .stats_setting
                .pixela_token
                .as_ref()
                .map(|t| "•".repeat(t.len().min(max_text_len - 1)))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "-".to_string()),
        };
        Paragraph::new("Api key: ")
            .right_aligned()
            .render(api_cols[0], buf);
        Paragraph::new(Text::styled(
            token_display,
            SettingsModal::highlight_selected(selected_setting, 2, mode),
        ))
        .left_aligned()
        .render(api_cols[1], buf);
    }
    fn highlight_selected(selected_num: u8, setting_num: u8, current_mode: Mode) -> Style {
        if setting_num == selected_num {
            match current_mode {
                Mode::Normal => Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                Mode::Input => Style::default().fg(BLUE),
            }
        } else {
            Style::default().fg(Color::White)
        }
    }
}
