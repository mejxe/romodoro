use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    prelude::Buffer,
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Clear, ListState, Padding, Paragraph, Widget, Wrap},
};

use crate::{
    ui::{BLUE, GREEN},
    utils::tabs::Tabs,
};

use super::{popup::popup_area, YELLOW};

pub fn render_scroll_indicators(
    list_area: Rect,
    buf: &mut Buffer,
    total_items: usize,
    list_state: &ListState,
    color: Color,
) {
    let list_inner = list_area.inner(Margin {
        horizontal: 0,
        vertical: 1,
    });
    let offset = list_state.offset();
    let list_height = list_inner.height as usize;
    let _selected = list_state.selected().unwrap_or(0);

    if offset > 0 {
        let up_arrow = Paragraph::new("▲")
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        let arrow_area = Rect {
            x: list_area.x,
            y: list_area.y,
            width: list_area.width,
            height: 1,
        };
        up_arrow.render(arrow_area, buf);
    }

    if total_items > list_height && offset < total_items.saturating_sub(list_height) {
        let down_arrow = Paragraph::new("▼")
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        let arrow_area = Rect {
            x: list_area.x,
            y: list_area.y + list_area.height - 1,
            width: list_area.width,
            height: 1,
        };
        down_arrow.render(arrow_area, buf);
    }
}

#[derive(Debug)]
pub enum Modal {
    HelperModal,
    SettingsModal,
}
pub struct HelpModal {
    current_tab: Tabs,
}
impl HelpModal {
    pub(crate) fn new(current_tab: Tabs) -> Self {
        Self { current_tab }
    }
}
impl Widget for HelpModal {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .title("Help")
            .border_style(Style::new().fg(YELLOW))
            .padding(Padding::symmetric(2, 1));
        let modal_area = popup_area(area, 40, 50);
        Clear.render(modal_area, buf);
        let inner = block.inner(modal_area);
        block.render(modal_area, buf);
        match self.current_tab {
            Tabs::TimerTab => HelpModal::render_timer_help(inner, buf),
            Tabs::StatsTab => HelpModal::render_stats_help(inner, buf),
        }
    }
}

impl HelpModal {
    fn render_timer_help(area: Rect, buf: &mut Buffer) {
        let layout =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
        let timer_header = Line::raw("Timer").centered().bold().fg(BLUE);
        let logging_header = Line::raw("Logging").centered().bold().fg(GREEN);
        let timer_text = Text::from(vec![
            timer_header,
            "Choose between Pomodoro and Flowmodoro modes.".into(),
            "\n".into(),
            "Space - Start/Stop".into(),
            "r - Reset Timer".into(),
            "x - Start break (Flowmodoro)".into(),
        ]);
        let logging_text = Text::from(vec![
            logging_header,
            "If you have stats on your sessions will be logged in the stats tab.".into(),
            "\n".into(),
            "• For Pomodoro - when finishing a single iteration".into(),
            "• For Flowmodoro - when starting a break, or hitting a minimum logging threshold"
                .into(),
        ])
        .left_aligned();
        let logging_paragraph = Paragraph::new(logging_text).wrap(Wrap { trim: true });
        timer_text.render(layout[0], buf);
        logging_paragraph.render(layout[1], buf);
    }
    fn render_stats_help(area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Percentage(45),
            Constraint::Percentage(25),
        ])
        .split(area);
        let simple_stats_header = Line::raw("Simple stats").centered().bold().fg(BLUE);
        let complex_stats_header = Line::raw("Complex stats").centered().bold().fg(GREEN);
        let graph_header = Line::raw("Charts").centered().bold().fg(YELLOW);
        let simple_text = Text::from(vec![
            simple_stats_header,
            "Provide a username to track your sessions locally as pixels.".into(),
            "Pixel: (Date | Session Length).".into(),
        ]);
        let complex_text = Text::from(vec![
            complex_stats_header,
            "Provide a Pixe.la api key and log in to track your sessions as uploadable pixels."
                .into(),
            "Pixel: (Subject* | Date | Session Length).".into(),
            "Pixel can be pushed to Pixe.la if the graph can hold Pixel's value.".into(),
            "\n".into(),
            "Subject - Pixe.la Graph".into(),
        ])
        .left_aligned();
        let graph_text = Text::from(vec![
            graph_header,
            "Display your Pixe.la graph as a histogram by pressing 'G'.".into(),
        ]);
        let simple_paragraph = Paragraph::new(simple_text).wrap(Wrap { trim: true });
        let complex_paragraph = Paragraph::new(complex_text).wrap(Wrap { trim: true });
        let chart_paragraph = Paragraph::new(graph_text).wrap(Wrap { trim: true });
        simple_paragraph.render(layout[0], buf);
        complex_paragraph.render(layout[1], buf);
        chart_paragraph.render(layout[2], buf);
    }
}
