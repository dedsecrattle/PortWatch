use crate::app::AppState;
use crate::models::ConnectionState;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let filtered_ports = state.filtered_ports();
    
    // Create table state for scrolling
    let mut table_state = TableState::default();
    table_state.select(Some(state.selected_index));

    let header = Row::new(vec![
        Cell::from("ADDRESS"),
        Cell::from("PORT"),
        Cell::from("PROTO"),
        Cell::from("STATE"),
        Cell::from("PROCESS"),
        Cell::from("PID"),
    ])
    .style(theme.header)
    .bottom_margin(1);

    let rows: Vec<Row> = filtered_ports
        .iter()
        .map(|port| {
            let state_style = match port.state {
                ConnectionState::Listen => theme.listen,
                ConnectionState::Established => theme.established,
                _ => theme.other_state,
            };

            let protocol_style = match port.protocol {
                crate::models::Protocol::Tcp => theme.tcp,
                crate::models::Protocol::Udp => theme.udp,
            };

            Row::new(vec![
                Cell::from(if port.local_addr == "0.0.0.0" || port.local_addr == "::" {
                    "*".to_string()
                } else {
                    port.local_addr.clone()
                }).style(theme.address),
                Cell::from(port.local_port.to_string()).style(theme.port),
                Cell::from(port.protocol.to_string()).style(protocol_style),
                Cell::from(port.state.to_string()).style(state_style),
                Cell::from(
                    port.process_name
                        .as_deref()
                        .unwrap_or("-")
                        .to_string(),
                ).style(theme.process_name),
                Cell::from(
                    port.pid
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                ).style(theme.pid),
            ])
        })
        .collect();

    let title = if state.filter.is_empty() {
        format!("Ports & Connections ({} total)", filtered_ports.len())
    } else {
        format!(
            "Ports & Connections (filter: '{}', {} matches)",
            state.filter,
            filtered_ports.len()
        )
    };

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Length(15),
            ratatui::layout::Constraint::Length(6),
            ratatui::layout::Constraint::Length(6),
            ratatui::layout::Constraint::Length(12),
            ratatui::layout::Constraint::Min(12),
            ratatui::layout::Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(ratatui::text::Span::styled(
                format!(" {} ", title),
                theme.title,
            ))
            .borders(Borders::ALL)
            .border_style(theme.border_focused),
    )
    .row_highlight_style(theme.selected);

    f.render_stateful_widget(table, area, &mut table_state);
}
