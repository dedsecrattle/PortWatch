use crate::app::AppState;
use crate::events::EventHandler;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState, event_handler: &EventHandler, theme: &Theme) {
    let keybindings = if event_handler.is_filter_mode() {
        vec![
            Span::styled("Filter mode: ", theme.footer_key),
            Span::styled("Type to filter | Enter to apply | Esc to cancel", theme.footer_text),
        ]
    } else {
        vec![
            Span::styled("↑↓", theme.footer_key),
            Span::styled(" navigate  ", theme.footer_text),
            Span::styled("Enter", theme.footer_key),
            Span::styled(" details  ", theme.footer_text),
            Span::styled("/", theme.footer_key),
            Span::styled(" filter  ", theme.footer_text),
            Span::styled("a", theme.footer_key),
            Span::styled(" alerts  ", theme.footer_text),
            Span::styled("E", theme.footer_key),
            Span::styled(" rules  ", theme.footer_text),
            Span::styled("k", theme.footer_key),
            Span::styled(" kill  ", theme.footer_text),
            Span::styled("r", theme.footer_key),
            Span::styled(" refresh  ", theme.footer_text),
            Span::styled("?", theme.footer_key),
            Span::styled(" help  ", theme.footer_text),
            Span::styled("q", theme.footer_key),
            Span::styled(" quit", theme.footer_text),
        ]
    };

    let content = if let Some(ref msg) = state.status_message {
        Line::from(
            [
                vec![
                    Span::styled("● ", theme.listen),
                    Span::styled(msg, theme.footer_text),
                    Span::styled(" | ", theme.footer_text),
                ],
                keybindings,
            ]
            .concat(),
        )
    } else {
        Line::from(keybindings)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .style(theme.status_bar);

    let paragraph = Paragraph::new(content)
        .block(block);

    f.render_widget(paragraph, area);
}
