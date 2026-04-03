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
            Span::styled("Filter mode: ", theme.value_highlight),
            Span::styled("Type to filter | Enter to apply | Esc to cancel", theme.value),
        ]
    } else {
        vec![
            Span::styled("↑↓", theme.value_highlight),
            Span::styled(" navigate  ", theme.normal),
            Span::styled("Enter", theme.value_highlight),
            Span::styled(" details  ", theme.normal),
            Span::styled("/", theme.value_highlight),
            Span::styled(" filter  ", theme.normal),
            Span::styled("k", theme.value_highlight),
            Span::styled(" kill  ", theme.normal),
            Span::styled("K", theme.value_highlight),
            Span::styled(" force  ", theme.normal),
            Span::styled("r", theme.value_highlight),
            Span::styled(" refresh  ", theme.normal),
            Span::styled("?", theme.value_highlight),
            Span::styled(" help  ", theme.normal),
            Span::styled("q", theme.value_highlight),
            Span::styled(" quit", theme.normal),
        ]
    };

    let content = if let Some(ref msg) = state.status_message {
        Line::from(vec![
            vec![
                Span::styled("● ", theme.listen),
                Span::styled(msg, theme.value),
                Span::styled(" | ", theme.normal),
            ],
            keybindings,
        ].concat())
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
