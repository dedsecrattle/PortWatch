use crate::app::AppState;
use crate::events::EventHandler;
use crate::ui::{alerts, details, footer, ports_table, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_main(f: &mut Frame, state: &AppState, event_handler: &EventHandler, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    if state.show_alerts {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(chunks[0]);

        ports_table::render(f, main_chunks[0], state, theme);
        details::render(f, main_chunks[1], state, theme);
        alerts::render(f, main_chunks[2], state, theme);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(chunks[0]);

        ports_table::render(f, main_chunks[0], state, theme);
        details::render(f, main_chunks[1], state, theme);
    }
    
    footer::render(f, chunks[1], state, event_handler, theme);
}

pub fn render_help(f: &mut Frame, theme: &Theme) {
    let help_text = vec![
        "PortScope - Keyboard Shortcuts",
        "",
        "Navigation:",
        "  ↑           Move up",
        "  ↓           Move down",
        "  Enter       View process details",
        "",
        "Actions:",
        "  r           Refresh port list",
        "  /           Start filter mode",
        "  Esc         Clear filter",
        "  k           Graceful stop (SIGTERM)",
        "  K           Force kill (SIGKILL)",
        "",
        "Other:",
        "  a           Toggle alerts panel",
        "  ?           Toggle this help",
        "  q / Ctrl+C  Quit",
        "",
        "Press ? to close this help screen",
    ];

    let help_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(theme.border);

    let help_paragraph = Paragraph::new(help_text.join("\n"))
        .block(help_block)
        .style(theme.normal);

    let area = centered_rect(60, 70, f.area());
    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(help_paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
