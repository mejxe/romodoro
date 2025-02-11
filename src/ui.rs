use crate::app::App;
use crate::romodoro::Pomodoro;
use crate::settings::SettingsTab;
use ratatui::{self, buffer::{self, Buffer}, layout::{Alignment, Constraint, Direction, Layout, Margin, Rect}, style::{palette::tailwind, Color, Modifier, Style, Styled, Stylize}, symbols::{self, border}, text::{Line, Span, Text}, widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph, Tabs, Widget}, DefaultTerminal, Frame};

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tabs = ["Pomodoro Timer", "Settings", "Stats"];
        let tab_titles: Vec<Span> = tabs.iter().map(|t| Span::styled(*t, Style::default().fg(Color::White))).collect();
        let selected_tab = self.get_selected_tab();

        let tabs_widget = Tabs::new(tab_titles)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title(" Menu ")
                .border_style(Style::default().fg(Color::Rgb(215,153,33)))) // Match Pomodoro timer border color
            .highlight_style(Style::default().fg(Color::Rgb(240,94,90))) // Pomodoro color
            .select(selected_tab);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab titles
                Constraint::Min(1),   // Main content area
                Constraint::Max(1), // footer
            ])
            .split(area);

        let tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Max(38), // Limit width of tabs
                Constraint::Min(1),
            ])
            .split(layout[0]);

        tabs_widget.render(tab_layout[0], buf);

        match selected_tab {
            0 => self.get_pomodoro_ref().render(layout[1], buf),
            1 => self.get_settings_ref().render(layout[1], buf),
            2 => self.render_stats(layout[1], buf),
            _ => {}
        }
    self.render_footer(layout[2], buf);
    }
}
impl App {
    pub fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let footer_text = match self.get_selected_tab() {
            0 => "Space: Start/Stop | Tab: Next Tab | Q: Quit",
            1 => "↑↓: Select | ←→: Adjust Value | Space: Confirm | Tab: Next Tab | Q: Quit",
            _ => "Tab: Next Tab | Q: Quit",
        };

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC));

        footer.render(area, buf);
    }
    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new("Stats Page (Placeholder)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightGreen));
        text.render(area, buf);
    }
    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}


impl Widget for &Pomodoro {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let time = self.timer.get_timeleft();
        let total_time = self.timer.get_total_time();
        let elapsed_time = self.timer.get_total_elapsed_time();
        let now_text = format!("Now: {}", self.timer.get_current_state());
        let progress = (elapsed_time) as f64 / total_time as f64;
        let iterations_text = format!("{}/{} iterations", self.timer.get_iteration(), self.timer.get_total_iterations());

        let outer_block = Block::default()
            .title(" Pomodoro Timer ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::Rgb(215,153,33))); // Gruvbox dark border

        let timer_text = Paragraph::new(format_ascii_time(&format!("{:02}:{:02}:{:02}", time/3600,(time%3600)/60, time%60)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::NONE));

        let now_paragraph_style = match self.timer.get_current_state() {
            crate::timer::PomodoroState::Work(_) => Style::default().fg(Color::LightBlue).add_modifier(Modifier::DIM),
            crate::timer::PomodoroState::Break(_) => Style::default().fg(Color::LightGreen).add_modifier(Modifier::DIM),
        };
        let now_paragraph = Paragraph::new(now_text)
            .alignment(Alignment::Center)
            .style(now_paragraph_style);

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
                Constraint::Max(9), // Bigger timer text
                Constraint::Length(2), // Smaller "{}/{} iterations"
                Constraint::Length(3), // Progress bar
                Constraint::Max(5),
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
    
impl Widget for &SettingsTab {
        fn render(self, area: Rect, buf: &mut Buffer) {

            let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35), // Empty space (Top)
            Constraint::Percentage(30),      // Settings box height
            Constraint::Percentage(35), // Empty space (Bottom)
        ])
        .split(area);

    let centered_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(37), // Empty space (Left)
            Constraint::Percentage(35), // Settings box width
            Constraint::Percentage(37), // Empty space (Right)
        ])
        .split(outer_layout[1]); // Centered in vertical space

    let settings_area = centered_layout[1];

    let settings_box = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(99, 150, 99)));

    let inner_area = settings_box.inner(settings_area);
    settings_box.render(settings_area, buf);
            let settings_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Max(3),
                    Constraint::Max(3),
                    Constraint::Max(3),
                    Constraint::Max(3),
                    Constraint::Max(3),
                    Constraint::Percentage(20),
                ])
                .split(inner_area);
        let work_time_text = Paragraph::new("Work Time")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)).add_modifier(Modifier::BOLD);

        let break_time_text = Paragraph::new("Break Time")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)).add_modifier(Modifier::BOLD);

            let work_time_style = if self.selected_setting == 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen)
            };

            let break_time_style = if self.selected_setting == 1 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen)
            };

            let work_time_value = Paragraph::new(format!("{} min", (self.work_time/60)))
                .alignment(Alignment::Center)
                .style(work_time_style);

            let break_time_value = Paragraph::new(format!("{} min", self.break_time/60))
                .alignment(Alignment::Center)
                .style(break_time_style);

            let iterations_style = if self.selected_setting == 2 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen)
            };
            let iterations_text = Paragraph::new("Iterations")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White)).add_modifier(Modifier::BOLD);

            let iterations_value = Paragraph::new(format!("{} iters", self.iterations))
                .alignment(Alignment::Center)
                .style(iterations_style);

            work_time_text.render(settings_layout[1], buf);
            work_time_value.render(settings_layout[2], buf);
            break_time_text.render(settings_layout[3], buf);
            break_time_value.render(settings_layout[4], buf);
            iterations_text.render(settings_layout[5], buf);
            iterations_value.render(settings_layout[6], buf);
    }
}
fn format_ascii_time(input: &str) -> String {
    let mut output = vec![String::new(); 7]; // 7 lines per character

    for ch in input.chars() {
        let index = match ch {
            '0'..='9' => ch as usize - '0' as usize,
            ':' => 10,
            _ => continue,
        };
        
        let ascii_lines: Vec<&str> = ASCII_NUMBERS[index].lines().collect();
        
        for (i, line) in ascii_lines.iter().enumerate() {
            output[i].push_str(line);
            output[i].push_str("  "); // Space between numbers
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

#[cfg(test)]
mod test {
    use crate::ui::format_ascii_time;

    #[test]
    fn ascii_text_works() {
        let time = "01:32:29";
        println!("{}",format_ascii_time(time));
    }
}
