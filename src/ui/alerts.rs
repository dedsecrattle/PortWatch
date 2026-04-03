use crate::alerts::AlertSeverity;
use crate::app::AppState;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Wrap, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let alerts = state.alert_manager.get_recent_alerts(20);
    
    if alerts.is_empty() {
        let block = Block::default()
            .title(Span::styled(" Alerts (0) ", theme.title))
            .borders(Borders::ALL)
            .border_style(theme.border_focused);
        
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("No alerts yet.", theme.value)),
            Line::from(""),
            Line::from(Span::styled("Alerts will appear here when:", theme.normal)),
            Line::from(Span::styled("• A port opens/closes", theme.normal)),
            Line::from(Span::styled("• External connections detected", theme.normal)),
            Line::from(Span::styled("• CPU/Memory thresholds exceeded", theme.normal)),
        ]);
        
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = alerts
            .iter()
            .rev()
            .map(|alert| {
                let severity_style = match alert.severity {
                    AlertSeverity::Info => Style::default().fg(Color::Cyan),
                    AlertSeverity::Warning => Style::default().fg(Color::Yellow),
                    AlertSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                };
                
                let time = alert.timestamp.format("%H:%M:%S").to_string();
                let severity_label = match alert.severity {
                    AlertSeverity::Info => "INFO",
                    AlertSeverity::Warning => "WARN",
                    AlertSeverity::Critical => "CRIT",
                };
                
                // Wrap message text to fit panel width
                let max_width = (area.width.saturating_sub(4)) as usize;
                let mut all_lines = vec![];
                
                // Add header line
                all_lines.push(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", time),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{} ", severity_label),
                        severity_style,
                    ),
                ]));
                
                // Wrap message text
                let words: Vec<&str> = alert.message.split_whitespace().collect();
                let mut current_line = String::new();
                
                for word in words {
                    if current_line.len() + word.len() + 1 > max_width && !current_line.is_empty() {
                        all_lines.push(Line::from(
                            Span::styled(format!("  {}", current_line), theme.normal)
                        ));
                        current_line.clear();
                    }
                    
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                }
                
                if !current_line.is_empty() {
                    all_lines.push(Line::from(
                        Span::styled(format!("  {}", current_line), theme.normal)
                    ));
                }
                
                // Add separator line
                all_lines.push(Line::from(""));
                
                ListItem::new(all_lines)
            })
            .collect();
        
        let block = Block::default()
            .title(Span::styled(
                format!(" Alerts ({}) ", alerts.len()),
                theme.title,
            ))
            .borders(Borders::ALL)
            .border_style(theme.border_focused);
        
        let list = List::new(items).block(block);
        
        f.render_widget(list, area);
    }
}
