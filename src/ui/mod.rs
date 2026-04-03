mod theme;
mod layout;
mod ports_table;
mod details;
mod footer;
mod alerts;
pub mod alert_rules;

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

    if state.alert_rules.open {
        let area = layout::centered_rect(88, 85, f.area());
        alert_rules::render_overlay(
            f,
            area,
            state.alert_manager.get_rules(),
            &state.alert_rules,
        );
    }
}
