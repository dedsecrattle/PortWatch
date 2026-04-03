mod theme;
mod layout;
mod ports_table;
mod details;
mod footer;
mod alerts;

pub use theme::Theme;
use crate::app::AppState;
use crate::events::EventHandler;
use ratatui::Frame;

pub fn render(f: &mut Frame, state: &AppState, event_handler: &EventHandler) {
    let theme = Theme::default();
    
    if state.show_help {
        layout::render_help(f, &theme);
    } else {
        layout::render_main(f, state, event_handler, &theme);
    }
}
