use crate::app::AppState;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let content = if let Some(selected_port) = state.get_selected_port() {
        let mut lines = vec![];
        
        // Port connection info
        lines.push(format!("Local: {}:{}", selected_port.local_addr, selected_port.local_port));
        if let (Some(ref remote_addr), Some(remote_port)) = (&selected_port.remote_addr, selected_port.remote_port) {
            lines.push(format!("Remote: {}:{}", remote_addr, remote_port));
        }
        lines.push(String::new());
        
        // Process details if available
        if let Some(ref details) = state.process_details {
            lines.push(format!("PID: {}", details.pid));
            lines.push(format!("Name: {}", details.name));

            if let Some(ref exe) = details.exe {
                lines.push(format!("Executable: {}", exe.display()));
            } else if let Some(ref port_exe) = selected_port.exe {
                lines.push(format!("Executable: {}", port_exe.display()));
            }

            if let Some(ref cwd) = details.cwd {
                lines.push(format!("Working Dir: {}", cwd.display()));
            }

            if let Some(ppid) = details.parent_pid {
                lines.push(format!("Parent PID: {}", ppid));
            }

            if let Some(ref user) = details.user {
                lines.push(format!("User: {}", user));
            } else if let Some(ref port_user) = selected_port.user {
                lines.push(format!("User: {}", port_user));
            }

            lines.push(format!("Memory: {}", details.format_memory()));
            lines.push(format!("CPU: {:.1}%", details.cpu_percent));
            lines.push(format!("Uptime: {}", details.format_uptime()));
            lines.push(String::new());

            if !details.cmdline.is_empty() {
                lines.push("Command:".to_string());
                let cmd = details.cmdline.join(" ");
                if cmd.len() > 80 {
                    lines.push(format!("  {}...", &cmd[..77]));
                } else {
                    lines.push(format!("  {}", cmd));
                }
                lines.push(String::new());
            } else if !selected_port.cmdline.is_empty() {
                lines.push("Command:".to_string());
                let cmd = selected_port.cmdline.join(" ");
                if cmd.len() > 80 {
                    lines.push(format!("  {}...", &cmd[..77]));
                } else {
                    lines.push(format!("  {}", cmd));
                }
                lines.push(String::new());
            }

            if !details.env_preview.is_empty() {
                lines.push("Environment (preview):".to_string());
                for (key, value) in details.env_preview.iter().take(5) {
                    let display_value = if value.len() > 40 {
                        format!("{}...", &value[..37])
                    } else {
                        value.clone()
                    };
                    lines.push(format!("  {}={}", key, display_value));
                }
            }
        } else if selected_port.pid.is_some() {
            lines.push("Loading process details...".to_string());
        } else {
            lines.push("No process associated with this port".to_string());
        }

        lines.join("\n")
    } else {
        "Select a port to view details".to_string()
    };

    let block = Block::default()
        .title("Process Details")
        .borders(Borders::ALL)
        .border_style(theme.border);

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(theme.normal)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
