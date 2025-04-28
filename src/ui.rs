use crate::app::App;
use crate::romodoro::Pomodoro;
use crate::settings::SettingsTab;
use ratatui::{
    self,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Tabs, Widget},
    Frame,
};
// gruvbox theme
const YELLOW: Color = Color::Rgb(215, 153, 33);
const BLUE: Color = Color::Rgb(69, 133, 136);
const GREEN: Color = Color::Rgb(142, 192, 124);
const RED: Color = Color::Rgb(204, 36, 29);
const BG: Color = Color::Rgb(40, 40, 40);

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tabs = ["Pomodoro Timer", "Settings", "Stats"];
        let tab_titles: Vec<Span> = tabs
            .iter()
            .map(|t| Span::styled(*t, Style::default().fg(Color::White)))
            .collect();
        let selected_tab = self.get_selected_tab();

        let tabs_widget = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Menu ")
                    .border_style(Style::default().fg(YELLOW)),
            )
            .highlight_style(Style::default().fg(Color::Rgb(240, 94, 90)))
            .select(selected_tab);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab titles
                Constraint::Min(1),    // Main content area
                Constraint::Max(1),    // footer
            ])
            .split(area);

        let tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(38), Constraint::Min(1)])
            .split(layout[0]);

        tabs_widget.render(tab_layout[0], buf);

        let popup_area = centered_rect(40, 30, area);
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(30)])
            .vertical_margin(0)
            .split(popup_area);
        let popup_yes_no_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .horizontal_margin(0)
            .split(popup_layout[1]);
        let yes_paragraph = Paragraph::new("<y>es")
            .alignment(Alignment::Center)
            .centered()
            .style(Style::default().fg(GREEN))
            .block(
                Block::default()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            );
        let no_paragraph = Paragraph::new("<n>o")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RED))
            .block(
                Block::default()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            );
        let question_paragraph = Paragraph::new("This will reset your current timer!")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .title("You sure?"),
            );
        match selected_tab {
            0 => self.get_pomodoro_ref().render(layout[1], buf),
            1 => self.get_settings_ref().borrow().render(layout[1], buf),
            2 => self.render_stats(layout[1], buf),
            _ => {}
        }
        if self.get_show_popup() && selected_tab == 1 {
            Clear.render(area, buf);
            popup_block.render(popup_area, buf);
            question_paragraph.render(popup_layout[0], buf);
            yes_paragraph.render(popup_yes_no_layout[0], buf);
            no_paragraph.render(popup_yes_no_layout[1], buf);
            return;
        }
        self.render_footer(layout[2], buf);
    }
}
impl App {
    pub fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let footer_text = match self.get_selected_tab() {
            0 => "Space: Start/Stop | Tab: Next Tab | Q: Quit",
            1 => "↑↓: Select | ←→: Adjust Value | Space: Confirm | Tab: Next Tab | r: Restore Defaults | Q: Quit |" ,
            _ => "Tab: Next Tab | Q: Quit",
        };

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
        let text = Paragraph::new("Stats Page (WIP)".to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightGreen));
        text.render(area, buf);
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}
//impl Widget for &Pomodoro {
//    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
//        let time = self.timer.get_timeleft();
//        let total_time = self.timer.get_total_time();
//        let elapsed_time = self.timer.get_total_elapsed_time();
//        let now_text = format!("Now: {}", self.timer.get_current_state());
//        let progress = (elapsed_time) as f64 / total_time as f64;
//        let iterations_text = format!(
//            "{}/{} iterations",
//            self.timer.get_iteration(),
//            self.timer.get_total_iterations()
//        );
//
//        let outer_block = Block::default()
//            .title(" Pomodoro Timer ")
//            .borders(Borders::ALL)
//            .border_type(BorderType::Double)
//            .style(Style::default().fg(Color::Rgb(215, 153, 33))); // Gruvbox dark border
//
//        let (timer_style, text_of_timer) = match self
//            .get_setting_ref()
//            .borrow()
//            .ui_settings
//            .hide_work_countdown
//        {
//            true if self.timer.get_running() => (
//                Style::default().fg(Color::LightRed),
//                format_ascii_time("00:00:00"),
//            ),
//            true => (
//                Style::default().fg(Color::LightRed),
//                format_ascii_time(&format!(
//                    "{:02}:{:02}:{:02}",
//                    time / 3600,
//                    (time % 3600) / 60,
//                    time % 60
//                )),
//            ),
//            false => (
//                Style::default()
//                    .fg(Color::LightYellow)
//                    .add_modifier(Modifier::BOLD),
//                format_ascii_time(&format!(
//                    "{:02}:{:02}:{:02}",
//                    time / 3600,
//                    (time % 3600) / 60,
//                    time % 60
//                )),
//            ),
//        };
//        let timer_text = Paragraph::new(text_of_timer)
//            .alignment(Alignment::Center)
//            .style(timer_style)
//            .block(Block::default().borders(Borders::NONE));
//
//        let now_paragraph_style = match self.timer.get_current_state() {
//            crate::timer::PomodoroState::Work(_) => Style::default()
//                .fg(Color::LightBlue)
//                .add_modifier(Modifier::DIM),
//            crate::timer::PomodoroState::Break(_) => Style::default()
//                .fg(Color::LightGreen)
//                .add_modifier(Modifier::DIM),
//        };
//        let now_paragraph = Paragraph::new(now_text)
//            .alignment(Alignment::Center)
//            .style(now_paragraph_style);
//
//        let count_paragraph = Paragraph::new(iterations_text)
//            .alignment(Alignment::Center)
//            .style(
//                Style::default()
//                    .fg(Color::Rgb(69, 133, 136))
//                    .add_modifier(Modifier::ITALIC),
//            );
//
//        let gauge = Gauge::default()
//            .block(
//                Block::default()
//                    .borders(Borders::ALL)
//                    .border_style(Color::Rgb(99, 150, 99))
//                    .border_type(BorderType::Thick),
//            )
//            .gauge_style(Style::default().fg(Color::Rgb(85, 158, 85)))
//            .ratio(progress);
//
//        let layout = Layout::default()
//            .direction(Direction::Vertical)
//            .constraints([
//                Constraint::Length(2), //  "Now: {}"
//                Constraint::Length(2),
//                Constraint::Max(9),    //  timer text
//                Constraint::Length(2), //  "{}/{} iterations"
//                Constraint::Length(3), // Progress bar
//                Constraint::Max(5),
//                Constraint::Min(10), //  ASCII art
//            ])
//            .margin(2)
//            .split(area);
//
//        let gauge_layout = Layout::default()
//            .direction(Direction::Horizontal)
//            .constraints([
//                Constraint::Percentage(25),
//                Constraint::Percentage(50),
//                Constraint::Percentage(25),
//            ])
//            .split(layout[4]);
//        buf.set_style(area, Style::default().fg(Color::DarkGray));
//        outer_block.render(area, buf);
//
//        now_paragraph.render(layout[0], buf);
//        timer_text.render(layout[2], buf);
//        count_paragraph.render(layout[3], buf);
//        gauge.render(gauge_layout[1], buf);
//    }
//}
impl Widget for &Pomodoro {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let time = self.timer.get_timeleft();
        let total_time = self.timer.get_total_time();
        let elapsed_time = self.timer.get_total_elapsed_time();
        let now_text = format!("Now: {}", self.timer.get_current_state());
        let progress = (elapsed_time) as f64 / total_time as f64;
        let iterations_text = format!(
            "{}/{} iterations",
            self.timer.get_iteration(),
            self.timer.get_total_iterations()
        );

        let outer_block = Block::default()
            .title(" Pomodoro Timer ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(YELLOW)); // Gruvbox yellow for border

        // Determine timer display based on settings and state
        let (timer_style, text_of_timer) = match self
            .get_setting_ref()
            .borrow()
            .ui_settings
            .hide_work_countdown
        {
            true if self.timer.get_running() => {
                (Style::default().fg(RED), format_ascii_time("00:00:00"))
            }
            true => (
                Style::default().fg(RED),
                format_ascii_time(&format!(
                    "{:02}:{:02}:{:02}",
                    time / 3600,
                    (time % 3600) / 60,
                    time % 60
                )),
            ),
            false => (
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                format_ascii_time(&format!(
                    "{:02}:{:02}:{:02}",
                    time / 3600,
                    (time % 3600) / 60,
                    time % 60
                )),
            ),
        };

        let timer_text = Paragraph::new(text_of_timer)
            .alignment(Alignment::Center)
            .style(timer_style);

        // Style the state indicator based on current state
        let now_paragraph_style = match self.timer.get_current_state() {
            crate::timer::PomodoroState::Work(_) => Style::default().fg(BLUE),
            crate::timer::PomodoroState::Break(_) => Style::default().fg(GREEN),
        };

        let now_paragraph = Paragraph::new(now_text)
            .alignment(Alignment::Center)
            .style(now_paragraph_style);

        let count_paragraph = Paragraph::new(iterations_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(BLUE).add_modifier(Modifier::ITALIC));

        // Create gauge with proper title
        let gauge_style = match self.timer.get_current_state() {
            crate::timer::PomodoroState::Work(_) => BLUE,
            crate::timer::PomodoroState::Break(_) => GREEN,
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(gauge_style)
                    .title(" Progress ")
                    .title_alignment(Alignment::Center),
            )
            .gauge_style(Style::default().fg(gauge_style))
            .ratio(progress);

        // Calculate dynamic layout based on available space
        // This ensures proper spacing regardless of terminal size
        let available_height = area.height.saturating_sub(2); // Account for borders

        // Create a more balanced vertical layout
        let top_margin_percent = if available_height > 20 { 12 } else { 5 };
        let bottom_margin_percent = if available_height > 20 { 12 } else { 5 };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(top_margin_percent), // Top margin - dynamic
                Constraint::Length(1),                      // Now text
                Constraint::Length(5),                      // Small gap
                Constraint::Min(5),    // ASCII timer - give it space to breathe
                Constraint::Length(1), // Small gap
                Constraint::Length(1), // count
                Constraint::Length(2), // Small gap
                Constraint::Length(3), // Progress bar
                Constraint::Percentage(bottom_margin_percent), // Bottom margin - dynamic
            ])
            .split(area);

        // Create horizontal layout for centered gauge
        let gauge_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(layout[7]);

        // Render all elements
        outer_block.render(area, buf);
        now_paragraph.render(layout[1], buf);
        timer_text.render(layout[3], buf);
        count_paragraph.render(layout[5], buf);
        gauge.render(gauge_layout[1], buf);

        // Set background color while preserving existing styles
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).expect("should work lmao");
                if cell.style().bg == Some(Color::Reset) {
                    cell.set_style(cell.style().bg(BG));
                }
            }
        }
    }
}
impl Widget for &SettingsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a more compact vertical layout using percentages
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),  // Top gap
                Constraint::Percentage(35), // Pomodoro Settings
                Constraint::Percentage(5),  // Middle gap
                Constraint::Percentage(35), // Other Settings
                Constraint::Percentage(20), // Bottom gap -
            ])
            .split(area);

        // Create horizontal layout for centering panels using percentages
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left margin
                Constraint::Percentage(40), // Panel width - narrower
                Constraint::Percentage(30), // Right margin
            ]);

        let pomodoro_layout = horizontal_layout.split(outer_layout[1]);
        let other_layout = horizontal_layout.split(outer_layout[3]);

        let pomodoro_settings_area = pomodoro_layout[1];
        let other_settings_area = other_layout[1];

        // Create panel blocks with improved styling
        let pomodoro_box = Block::default()
            .title(" Pomodoro Settings ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BLUE));

        let other_settings_box = Block::default()
            .title(" Other Settings ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(RED));

        let pomodoro_inner_area = pomodoro_box.inner(pomodoro_settings_area);
        let other_inner_area = other_settings_box.inner(other_settings_area);

        // Create more compact inner layouts using percentages
        let pomodoro_inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),  // Top margin
                Constraint::Percentage(15), // Work Time label
                Constraint::Percentage(15), // Work Time value
                Constraint::Percentage(15), // Break Time label
                Constraint::Percentage(15), // Break Time value
                Constraint::Percentage(15), // Iterations label
                Constraint::Percentage(15), // Iterations value
                Constraint::Percentage(5),  // Bottom margin
            ])
            .split(pomodoro_inner_area);

        // Create more compact inner layouts for other settings
        let other_inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),  // Top margin
                Constraint::Percentage(15), // Pause setting label
                Constraint::Percentage(15), // Pause setting value
                Constraint::Percentage(15), // Hide clock label
                Constraint::Percentage(15), // Hide clock value
                Constraint::Percentage(5),  // Bottom margin
            ])
            .split(other_inner_area);

        // Render the panel blocks
        pomodoro_box.render(pomodoro_settings_area, buf);
        other_settings_box.render(other_settings_area, buf);

        // Create and render work time settings
        let work_time_text = Paragraph::new("Work Time")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .add_modifier(Modifier::BOLD);

        let work_time_value =
            Paragraph::new(format!("{} min", (self.timer_settings.work_time / 60)))
                .alignment(Alignment::Center)
                .style(self.highlight_selected(0));

        // Create and render break time settings
        let break_time_text = Paragraph::new("Break Time")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .add_modifier(Modifier::BOLD);

        let break_time_value =
            Paragraph::new(format!("{} min", self.timer_settings.break_time / 60))
                .alignment(Alignment::Center)
                .style(self.highlight_selected(1));

        // Create and render iterations settings
        let iterations_text = Paragraph::new("Iterations")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .add_modifier(Modifier::BOLD);

        let iterations_value = Paragraph::new(format!("{} iters", self.timer_settings.iterations))
            .alignment(Alignment::Center)
            .style(self.highlight_selected(2));

        // Render the pomodoro settings
        work_time_text.render(pomodoro_inner_layout[1], buf);
        work_time_value.render(pomodoro_inner_layout[2], buf);
        break_time_text.render(pomodoro_inner_layout[3], buf);
        break_time_value.render(pomodoro_inner_layout[4], buf);
        iterations_text.render(pomodoro_inner_layout[5], buf);
        iterations_value.render(pomodoro_inner_layout[6], buf);

        // Create and render pause setting
        let pause_change_state_text = Paragraph::new("Pause before new iteration")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .add_modifier(Modifier::BOLD);

        let pause_change_state_val = if self.ui_settings.pause_after_state_change {
            "yes"
        } else {
            "no"
        };

        let pause_change_state_value = Paragraph::new(pause_change_state_val.to_string())
            .alignment(Alignment::Center)
            .style(self.highlight_selected(3));

        // Create and render hide clock setting
        let hide_clock_text = Paragraph::new("Hide clock on work time")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .add_modifier(Modifier::BOLD);

        let hide_clock_val = if self.ui_settings.hide_work_countdown {
            "yes"
        } else {
            "no"
        };

        let hide_clock_value = Paragraph::new(hide_clock_val)
            .alignment(Alignment::Center)
            .style(self.highlight_selected(4));

        // Render the other settings
        pause_change_state_text.render(other_inner_layout[1], buf);
        pause_change_state_value.render(other_inner_layout[2], buf);
        hide_clock_text.render(other_inner_layout[3], buf);
        hide_clock_value.render(other_inner_layout[4], buf);

        // Apply background color to the entire area
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).expect("Should work");
                if cell.style().bg == Some(Color::Reset) {
                    cell.set_style(cell.style().bg(BG));
                }
            }
        }
    }
}
//impl Widget for &SettingsTab {
//    fn render(self, area: Rect, buf: &mut Buffer) {
//        let outer_layout = Layout::default()
//            .direction(Direction::Vertical)
//            .constraints([
//                Constraint::Percentage(10), // Empty space (Top)
//                Constraint::Percentage(40), // Pomodoro Settings
//                Constraint::Percentage(40), // Other Settings
//                Constraint::Percentage(10), // Empty space (Top)
//            ])
//            .split(area);
//
//        let centered_layout = Layout::default()
//            .direction(Direction::Horizontal)
//            .constraints([
//                Constraint::Percentage(33), // Empty space (Left)
//                Constraint::Percentage(33), // Empty space (Left)
//                Constraint::Percentage(33), // Empty space (Left)
//            ])
//            .split(outer_layout[1]);
//
//        let pomodoro_settings_area = centered_layout[1];
//
//        let centered_layout_other = Layout::default()
//            .direction(Direction::Horizontal)
//            .constraints([
//                Constraint::Percentage(33), // Empty space (Left)
//                Constraint::Percentage(33), // Empty space (Left)
//                Constraint::Percentage(33), // Empty space (Left)
//            ])
//            .split(outer_layout[2]);
//
//        let other_settings_area = centered_layout_other[1];
//
//        let pomodoro_box = Block::default()
//            .title(" Pomodoro Settings ")
//            .borders(Borders::ALL)
//            .border_type(BorderType::Double)
//            .border_style(Style::default().fg(BLUE));
//
//        let other_settings_box = Block::default()
//            .title(" Other Settings ")
//            .borders(Borders::ALL)
//            .border_type(BorderType::Double)
//            .border_style(Style::default().fg(RED));
//
//        let pomodoro_inner_area = pomodoro_box.inner(pomodoro_settings_area);
//        let other_inner_area = other_settings_box.inner(other_settings_area);
//
//        pomodoro_box.render(pomodoro_settings_area, buf);
//        other_settings_box.render(other_settings_area, buf);
//
//        let pomodoro_layout = Layout::default()
//            .direction(Direction::Vertical)
//            .constraints([
//                Constraint::Percentage(10),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Percentage(10),
//            ])
//            .split(pomodoro_inner_area);
//
//        let other_layout = Layout::default()
//            .direction(Direction::Vertical)
//            .constraints([
//                Constraint::Percentage(10),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Max(3),
//                Constraint::Percentage(10),
//            ])
//            .split(other_inner_area);
//
//        let work_time_text = Paragraph::new("Work Time")
//            .alignment(Alignment::Center)
//            .style(Style::default().fg(Color::White))
//            .add_modifier(Modifier::BOLD);
//
//        let break_time_text = Paragraph::new("Break Time")
//            .alignment(Alignment::Center)
//            .style(Style::default().fg(Color::White))
//            .add_modifier(Modifier::BOLD);
//
//        let work_time_value =
//            Paragraph::new(format!("{} min", (self.timer_settings.work_time / 60)))
//                .alignment(Alignment::Center)
//                .style(self.highlight_selected(0));
//
//        let break_time_value =
//            Paragraph::new(format!("{} min", self.timer_settings.break_time / 60))
//                .alignment(Alignment::Center)
//                .style(self.highlight_selected(1));
//
//        let iterations_text = Paragraph::new("Iterations")
//            .alignment(Alignment::Center)
//            .style(Style::default().fg(Color::White))
//            .add_modifier(Modifier::BOLD);
//
//        let iterations_value = Paragraph::new(format!("{} iters", self.timer_settings.iterations))
//            .alignment(Alignment::Center)
//            .style(self.highlight_selected(2));
//
//        work_time_text.render(pomodoro_layout[1], buf);
//        work_time_value.render(pomodoro_layout[2], buf);
//        break_time_text.render(pomodoro_layout[3], buf);
//        break_time_value.render(pomodoro_layout[4], buf);
//        iterations_text.render(pomodoro_layout[5], buf);
//        iterations_value.render(pomodoro_layout[6], buf);
//
//        let pause_change_state_text = Paragraph::new("Pause before starting new iteration")
//            .alignment(Alignment::Center)
//            .style(Style::default().fg(Color::White))
//            .add_modifier(Modifier::BOLD);
//
//        let pause_change_state_val = if self.ui_settings.pause_after_state_change {
//            "yes"
//        } else {
//            "no"
//        };
//        let pause_change_state_value = Paragraph::new(pause_change_state_val.to_string())
//            .alignment(Alignment::Center)
//            .style(self.highlight_selected(3));
//
//        let hide_clock_text = Paragraph::new("Hide clock on work time")
//            .alignment(Alignment::Center)
//            .style(Style::default().fg(Color::White))
//            .add_modifier(Modifier::BOLD);
//
//        let hide_clock_val = if self.ui_settings.hide_work_countdown {
//            "yes"
//        } else {
//            "no"
//        };
//        let hide_clock_value = Paragraph::new(format!("{}", hide_clock_val))
//            .alignment(Alignment::Center)
//            .style(self.highlight_selected(4));
//
//        pause_change_state_text.render(other_layout[1], buf);
//        pause_change_state_value.render(other_layout[2], buf);
//        hide_clock_text.render(other_layout[3], buf);
//        hide_clock_value.render(other_layout[4], buf);
//    }
//}

impl SettingsTab {
    fn highlight_selected(&self, setting_num: usize) -> Style {
        if setting_num == self.selected_setting {
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(GREEN)
        }
    }
}
fn format_ascii_time(input: &str) -> String {
    let mut output = vec![String::new(); 7];

    for ch in input.chars() {
        let index = match ch {
            '0'..='9' => ch as usize - '0' as usize,
            ':' => 10,
            _ => continue,
        };

        let ascii_lines: Vec<&str> = ASCII_NUMBERS[index].lines().collect();

        for (i, line) in ascii_lines.iter().enumerate() {
            output[i].push_str(line);
            output[i].push_str("  ");
        }
    }

    output.join("\n")
}

const ASCII_NUMBERS: [&str; 11] = [
    "  ███  \n █   █ \n█     █\n█     █\n█     █\n █   █ \n  ███  ", // 0
    "   █   \n  ██   \n █ █   \n   █   \n   █   \n   █   \n ████  ", // 1
    " ███   \n█   █  \n    █  \n   █   \n  █    \n █     \n█████  ", // 2
    " ███   \n█   █  \n    █  \n  ██   \n    █  \n█   █  \n ███   ", // 3
    "   ██  \n  █ █  \n █  █  \n█   █  \n█████  \n    █  \n    █  ", // 4
    "█████  \n█      \n████   \n    █  \n    █  \n█   █  \n ███   ", // 5
    "  ███  \n █     \n█      \n█ ███  \n█    █ \n █   █ \n  ███  ", // 6
    "█████  \n    █  \n   █   \n  █    \n █     \n █     \n █     ", // 7
    "  ███  \n █   █ \n █   █ \n  ███  \n █   █ \n █   █ \n  ███  ", // 8
    "  ███  \n █   █ \n █   █ \n  ████ \n     █ \n    ██ \n  ███  ", // 9
    "        \n   █    \n   █    \n        \n   █    \n   █    \n        ", // :
];
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

#[cfg(test)]
mod test {
    use crate::ui::format_ascii_time;

    #[test]
    fn ascii_text_works() {
        let time = "01:32:29";
        println!("{}", format_ascii_time(time));
    }
}
