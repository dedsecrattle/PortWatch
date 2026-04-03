use crate::app::AppState;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use ratatui::text::{Line, Span};

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let content = if let Some(selected_port) = state.get_selected_port() {
        let mut lines: Vec<Line> = vec![];
        
        // Port connection info with colors
        lines.push(Line::from(vec![
            Span::styled("Local: ", theme.label),
            Span::styled(
                format!("{}:{}", selected_port.local_addr, selected_port.local_port),
                theme.value_highlight,
            ),
        ]));
        
        if let (Some(ref remote_addr), Some(remote_port)) = (&selected_port.remote_addr, selected_port.remote_port) {
            lines.push(Line::from(vec![
                Span::styled("Remote: ", theme.label),
                Span::styled(
                    format!("{}:{}", remote_addr, remote_port),
                    theme.value_highlight,
                ),
            ]));
        }
        lines.push(Line::from(""));
        
        // Process details if available
        if let Some(ref details) = state.process_details {
            lines.push(Line::from(vec![
                Span::styled("PID: ", theme.label),
                Span::styled(details.pid.to_string(), theme.value),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Name: ", theme.label),
                Span::styled(&details.name, theme.value_highlight),
            ]));

            if let Some(ref exe) = details.exe {
                lines.push(Line::from(vec![
                    Span::styled("Executable: ", theme.label),
                    Span::styled(exe.display().to_string(), theme.value),
                ]));
            } else if let Some(ref port_exe) = selected_port.exe {
                lines.push(Line::from(vec![
                    Span::styled("Executable: ", theme.label),
                    Span::styled(port_exe.display().to_string(), theme.value),
                ]));
            }

            if let Some(ref cwd) = details.cwd {
                lines.push(Line::from(vec![
                    Span::styled("Working Dir: ", theme.label),
                    Span::styled(cwd.display().to_string(), theme.value),
                ]));
            }

            if let Some(ppid) = details.parent_pid {
                lines.push(Line::from(vec![
                    Span::styled("Parent PID: ", theme.label),
                    Span::styled(ppid.to_string(), theme.value),
                ]));
            }

            if let Some(ref user) = details.user {
                lines.push(Line::from(vec![
                    Span::styled("User: ", theme.label),
                    Span::styled(user, theme.value),
                ]));
            } else if let Some(ref port_user) = selected_port.user {
                lines.push(Line::from(vec![
                    Span::styled("User: ", theme.label),
                    Span::styled(port_user, theme.value),
                ]));
            }

            lines.push(Line::from(vec![
                Span::styled("Memory: ", theme.label),
                Span::styled(details.format_memory(), theme.value_highlight),
            ]));
            lines.push(Line::from(vec![
                Span::styled("CPU: ", theme.label),
                Span::styled(format!("{:.1}%", details.cpu_percent), theme.value_highlight),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Uptime: ", theme.label),
                Span::styled(details.format_uptime(), theme.value),
            ]));
            lines.push(Line::from(""));

            if !details.cmdline.is_empty() {
                lines.push(Line::from(Span::styled("Command:", theme.label)));
                let cmd = details.cmdline.join(" ");
                let cmd_display = if cmd.len() > 80 {
                    format!("  {}...", &cmd[..77])
                } else {
                    format!("  {}", cmd)
                };
                lines.push(Line::from(Span::styled(cmd_display, theme.value)));
                lines.push(Line::from(""));
            } else if !selected_port.cmdline.is_empty() {
                lines.push(Line::from(Span::styled("Command:", theme.label)));
                let cmd = selected_port.cmdline.join(" ");
                let cmd_display = if cmd.len() > 80 {
                    format!("  {}...", &cmd[..77])
                } else {
                    format!("  {}", cmd)
                };
                lines.push(Line::from(Span::styled(cmd_display, theme.value)));
                lines.push(Line::from(""));
            }

            if !details.env_preview.is_empty() {
                lines.push(Line::from(Span::styled("Environment (preview):", theme.label)));
                for (key, value) in details.env_preview.iter().take(5) {
                    let display_value = if value.len() > 40 {
                        format!("{}...", &value[..37])
                    } else {
                        value.clone()
                    };
                    lines.push(Line::from(vec![
                        Span::styled("  ", theme.normal),
                        Span::styled(format!("{}=", key), theme.label),
                        Span::styled(display_value, theme.value),
                    ]));
                }
            }
        } else if selected_port.pid.is_some() {
            lines.push(Line::from(Span::styled(
                "Loading process details...",
                theme.value,
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "No process associated with this port",
                theme.other_state,
            )));
        }

        lines
    } else {
        vec![Line::from(Span::styled(
            "Select a port to view details",
            theme.value,
        ))]
    };

    let block = Block::default()
        .title(ratatui::text::Span::styled(
            " Process Details ",
            theme.title,
        ))
        .borders(Borders::ALL)
        .border_style(theme.border_focused);

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
