use crate::app::AppState;
use crate::events::EventHandler;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState, event_handler: &EventHandler, theme: &Theme) {
    let keybindings = if event_handler.is_filter_mode() {
        "Filter mode: Type to filter | Enter to apply | Esc to cancel"
    } else {
        "↑↓ navigate  Enter details  / filter  k kill  K force  r refresh  ? help  q quit"
    };

    let status = if let Some(ref msg) = state.status_message {
        format!("{} | {}", msg, keybindings)
    } else {
        keybindings.to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border);

    let paragraph = Paragraph::new(status)
        .block(block)
        .style(theme.normal);

    f.render_widget(paragraph, area);
}
