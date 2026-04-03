//! Alert rule list + form overlay (CRUD).

use crate::alerts::{derive_rule_id, AlertCondition, AlertRule, AlertSeverity};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone)]
pub struct AlertRulesPanel {
    pub open: bool,
    pub selected: usize,
    pub form: Option<AlertRuleDraft>,
    /// Which field is focused in the form (0..)
    pub form_focus: usize,
    pub error: Option<String>,
}

impl Default for AlertRulesPanel {
    fn default() -> Self {
        Self {
            open: false,
            selected: 0,
            form: None,
            form_focus: 0,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionKind {
    PortOpened = 0,
    PortClosed = 1,
    PortRangeActivity = 2,
    ExternalConnection = 3,
    ProcessCpuThreshold = 4,
    ProcessMemoryThreshold = 5,
    UnknownProcessListening = 6,
}

impl ConditionKind {
    pub fn all() -> &'static [ConditionKind] {
        &[
            Self::PortOpened,
            Self::PortClosed,
            Self::PortRangeActivity,
            Self::ExternalConnection,
            Self::ProcessCpuThreshold,
            Self::ProcessMemoryThreshold,
            Self::UnknownProcessListening,
        ]
    }

    pub fn from_u8(n: u8) -> Self {
        Self::all()
            .get(n as usize)
            .copied()
            .unwrap_or(Self::PortOpened)
    }

}

#[derive(Debug, Clone)]
pub struct AlertRuleDraft {
    pub editing_index: Option<usize>,
    pub name: String,
    pub enabled: bool,
    pub severity: AlertSeverity,
    pub cooldown_seconds: String,
    pub condition_kind: ConditionKind,
    pub port: String,
    pub start_port: String,
    pub end_port: String,
    pub ip_pattern: String,
    pub exclude_private: bool,
    pub process_pattern: String,
    pub threshold_percent: String,
    pub threshold_mb: String,
}

impl Default for AlertRuleDraft {
    fn default() -> Self {
        Self {
            editing_index: None,
            name: String::new(),
            enabled: true,
            severity: AlertSeverity::Info,
            cooldown_seconds: "60".into(),
            condition_kind: ConditionKind::PortOpened,
            port: String::new(),
            start_port: "1".into(),
            end_port: "1024".into(),
            ip_pattern: ".*".into(),
            exclude_private: true,
            process_pattern: ".*".into(),
            threshold_percent: "50".into(),
            threshold_mb: "512".into(),
        }
    }
}

impl AlertRuleDraft {
    pub fn from_rule(rule: &AlertRule, index: usize) -> Self {
        let condition_kind = match &rule.condition {
            AlertCondition::PortOpened { .. } => ConditionKind::PortOpened,
            AlertCondition::PortClosed { .. } => ConditionKind::PortClosed,
            AlertCondition::PortRangeActivity { .. } => ConditionKind::PortRangeActivity,
            AlertCondition::ExternalConnection { .. } => ConditionKind::ExternalConnection,
            AlertCondition::ProcessCpuThreshold { .. } => ConditionKind::ProcessCpuThreshold,
            AlertCondition::ProcessMemoryThreshold { .. } => ConditionKind::ProcessMemoryThreshold,
            AlertCondition::UnknownProcessListening => ConditionKind::UnknownProcessListening,
        };
        let mut d = Self {
            editing_index: Some(index),
            name: rule.name.clone(),
            enabled: rule.enabled,
            severity: rule.severity,
            cooldown_seconds: rule.cooldown_seconds.to_string(),
            condition_kind,
            ..Self::default()
        };
        match &rule.condition {
            AlertCondition::PortOpened { port } => d.port = port.to_string(),
            AlertCondition::PortClosed { port } => d.port = port.to_string(),
            AlertCondition::PortRangeActivity { start_port, end_port } => {
                d.start_port = start_port.to_string();
                d.end_port = end_port.to_string();
            }
            AlertCondition::ExternalConnection {
                ip_pattern,
                exclude_private,
            } => {
                d.ip_pattern = ip_pattern.clone();
                d.exclude_private = *exclude_private;
            }
            AlertCondition::ProcessCpuThreshold {
                process_pattern,
                threshold_percent,
            } => {
                d.process_pattern = process_pattern.clone();
                d.threshold_percent = threshold_percent.to_string();
            }
            AlertCondition::ProcessMemoryThreshold {
                process_pattern,
                threshold_mb,
            } => {
                d.process_pattern = process_pattern.clone();
                d.threshold_mb = threshold_mb.to_string();
            }
            AlertCondition::UnknownProcessListening => {}
        }
        d
    }

    pub fn max_focus(&self) -> usize {
        match self.condition_kind {
            ConditionKind::PortOpened | ConditionKind::PortClosed => 5,
            ConditionKind::PortRangeActivity => 6,
            ConditionKind::ExternalConnection => 6,
            ConditionKind::ProcessCpuThreshold | ConditionKind::ProcessMemoryThreshold => 6,
            // Include field 4 (condition kind) so Tab can reach it and keys 0–6 can switch away.
            ConditionKind::UnknownProcessListening => 4,
        }
    }

    pub fn append_to_field(&mut self, focus: usize, c: char) {
        match focus {
            0 => self.name.push(c),
            1 | 2 | 4 => {}
            3 => {
                if c.is_ascii_digit() {
                    self.cooldown_seconds.push(c);
                }
            }
            _ => self.append_condition_field(focus, c),
        }
    }

    fn append_condition_field(&mut self, focus: usize, c: char) {
        match self.condition_kind {
            ConditionKind::PortOpened | ConditionKind::PortClosed => {
                if focus == 5 && (c.is_ascii_digit()) {
                    self.port.push(c);
                }
            }
            ConditionKind::PortRangeActivity => {
                if focus == 5 && c.is_ascii_digit() {
                    self.start_port.push(c);
                }
                if focus == 6 && c.is_ascii_digit() {
                    self.end_port.push(c);
                }
            }
            ConditionKind::ExternalConnection => {
                if focus == 5 {
                    self.ip_pattern.push(c);
                }
            }
            ConditionKind::ProcessCpuThreshold => {
                if focus == 5 {
                    self.process_pattern.push(c);
                }
                if focus == 6 && (c.is_ascii_digit() || c == '.') {
                    self.threshold_percent.push(c);
                }
            }
            ConditionKind::ProcessMemoryThreshold => {
                if focus == 5 {
                    self.process_pattern.push(c);
                }
                if focus == 6 && c.is_ascii_digit() {
                    self.threshold_mb.push(c);
                }
            }
            ConditionKind::UnknownProcessListening => {}
        }
    }

    pub fn backspace_field(&mut self, focus: usize) {
        let pop = |s: &mut String| {
            s.pop();
        };
        match focus {
            0 => pop(&mut self.name),
            3 => pop(&mut self.cooldown_seconds),
            5 => match self.condition_kind {
                ConditionKind::PortOpened | ConditionKind::PortClosed => pop(&mut self.port),
                ConditionKind::PortRangeActivity => pop(&mut self.start_port),
                ConditionKind::ExternalConnection => pop(&mut self.ip_pattern),
                ConditionKind::ProcessCpuThreshold | ConditionKind::ProcessMemoryThreshold => {
                    pop(&mut self.process_pattern)
                }
                ConditionKind::UnknownProcessListening => {}
            },
            6 => match self.condition_kind {
                ConditionKind::PortRangeActivity => pop(&mut self.end_port),
                ConditionKind::ProcessCpuThreshold => pop(&mut self.threshold_percent),
                ConditionKind::ProcessMemoryThreshold => pop(&mut self.threshold_mb),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn toggle_bool_field(&mut self, focus: usize) {
        match focus {
            1 => self.enabled = !self.enabled,
            6 if matches!(self.condition_kind, ConditionKind::ExternalConnection) => {
                self.exclude_private = !self.exclude_private;
            }
            _ => {}
        }
    }

    pub fn cycle_severity(&mut self) {
        self.severity = match self.severity {
            AlertSeverity::Info => AlertSeverity::Warning,
            AlertSeverity::Warning => AlertSeverity::Critical,
            AlertSeverity::Critical => AlertSeverity::Info,
        };
    }

    pub fn set_condition_kind(&mut self, n: u8) {
        self.condition_kind = ConditionKind::from_u8(n);
    }

    pub fn to_rule(&self) -> Result<AlertRule, String> {
        let cooldown: u64 = self
            .cooldown_seconds
            .trim()
            .parse()
            .map_err(|_| "cooldown must be a positive integer".to_string())?;
        if cooldown == 0 {
            return Err("cooldown must be > 0".into());
        }
        let name = self.name.trim();
        if name.is_empty() {
            return Err("name is required".into());
        }

        let condition = match self.condition_kind {
            ConditionKind::PortOpened => {
                let port: u16 = self
                    .port
                    .trim()
                    .parse()
                    .map_err(|_| "port must be a valid u16")?;
                AlertCondition::PortOpened { port }
            }
            ConditionKind::PortClosed => {
                let port: u16 = self
                    .port
                    .trim()
                    .parse()
                    .map_err(|_| "port must be a valid u16")?;
                AlertCondition::PortClosed { port }
            }
            ConditionKind::PortRangeActivity => {
                let start_port: u16 = self
                    .start_port
                    .trim()
                    .parse()
                    .map_err(|_| "start_port must be a valid u16")?;
                let end_port: u16 = self
                    .end_port
                    .trim()
                    .parse()
                    .map_err(|_| "end_port must be a valid u16")?;
                if start_port > end_port {
                    return Err("start_port must be <= end_port".into());
                }
                AlertCondition::PortRangeActivity {
                    start_port,
                    end_port,
                }
            }
            ConditionKind::ExternalConnection => {
                let ip_pattern = self.ip_pattern.trim().to_string();
                if ip_pattern.is_empty() {
                    return Err("ip_pattern is required".into());
                }
                let _ = regex::Regex::new(&ip_pattern).map_err(|e| e.to_string())?;
                AlertCondition::ExternalConnection {
                    ip_pattern,
                    exclude_private: self.exclude_private,
                }
            }
            ConditionKind::ProcessCpuThreshold => {
                let process_pattern = self.process_pattern.trim().to_string();
                if process_pattern.is_empty() {
                    return Err("process_pattern is required".into());
                }
                let _ = regex::Regex::new(&process_pattern).map_err(|e| e.to_string())?;
                let threshold_percent: f32 = self
                    .threshold_percent
                    .trim()
                    .parse()
                    .map_err(|_| "threshold_percent must be a number")?;
                AlertCondition::ProcessCpuThreshold {
                    process_pattern,
                    threshold_percent,
                }
            }
            ConditionKind::ProcessMemoryThreshold => {
                let process_pattern = self.process_pattern.trim().to_string();
                if process_pattern.is_empty() {
                    return Err("process_pattern is required".into());
                }
                let _ = regex::Regex::new(&process_pattern).map_err(|e| e.to_string())?;
                let threshold_mb: u64 = self
                    .threshold_mb
                    .trim()
                    .parse()
                    .map_err(|_| "threshold_mb must be a positive integer")?;
                AlertCondition::ProcessMemoryThreshold {
                    process_pattern,
                    threshold_mb,
                }
            }
            ConditionKind::UnknownProcessListening => AlertCondition::UnknownProcessListening,
        };

        let id = derive_rule_id(name, &condition);
        Ok(AlertRule {
            id,
            name: name.to_string(),
            condition,
            enabled: self.enabled,
            severity: self.severity,
            cooldown_seconds: cooldown,
        })
    }
}

pub fn render_overlay(f: &mut Frame, area: Rect, rules: &[AlertRule], panel: &AlertRulesPanel) {
    let block = Block::default()
        .title(" Alert rules (E close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(block, area);

    if let Some(ref draft) = panel.form {
        render_form(f, inner, draft, panel.form_focus, panel.error.as_deref());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(inner);

    let items: Vec<ListItem> = rules
        .iter()
        .enumerate()
        .map(|(i, r)| {
            // Enabled: checkmark in brackets; disabled: blank (reads clearer than `x`).
            let mark = if r.enabled { "✓" } else { " " };
            let line = format!("[{}] {} — {}", mark, r.name, condition_short(&r.condition));
            let style = if i == panel.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Rules (↑↓ n d t Enter) ")
            .borders(Borders::ALL),
    );
    f.render_widget(list, chunks[0]);

    let help = vec![
        Line::from(Span::styled(
            "n new  d delete  t toggle  Enter edit  Esc close",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        if rules.is_empty() {
            Line::from("No rules. Press n to add.")
        } else if panel.selected < rules.len() {
            Line::from(format!("Selected: {}", rules[panel.selected].id))
        } else {
            Line::from("")
        },
    ];
    let p = Paragraph::new(help)
        .block(Block::default().title(" Help ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(p, chunks[1]);
}

fn condition_short(c: &AlertCondition) -> String {
    match c {
        AlertCondition::PortOpened { port } => format!("port open {}", port),
        AlertCondition::PortClosed { port } => format!("port close {}", port),
        AlertCondition::PortRangeActivity { start_port, end_port } => {
            format!("range {}-{}", start_port, end_port)
        }
        AlertCondition::ExternalConnection { .. } => "external".into(),
        AlertCondition::ProcessCpuThreshold { .. } => "cpu".into(),
        AlertCondition::ProcessMemoryThreshold { .. } => "mem".into(),
        AlertCondition::UnknownProcessListening => "unknown listener".into(),
    }
}

fn render_form(
    f: &mut Frame,
    area: Rect,
    draft: &AlertRuleDraft,
    focus: usize,
    err: Option<&str>,
) {
    let sev = match draft.severity {
        AlertSeverity::Info => "Info",
        AlertSeverity::Warning => "Warning",
        AlertSeverity::Critical => "Critical",
    };
    let kind_line = format!(
        "Condition type: {} (keys 0-6)",
        match draft.condition_kind {
            ConditionKind::PortOpened => "PortOpened",
            ConditionKind::PortClosed => "PortClosed",
            ConditionKind::PortRangeActivity => "PortRange",
            ConditionKind::ExternalConnection => "External",
            ConditionKind::ProcessCpuThreshold => "Cpu",
            ConditionKind::ProcessMemoryThreshold => "Memory",
            ConditionKind::UnknownProcessListening => "UnknownListener",
        }
    );

    let mut lines: Vec<Line> = vec![
        Line::from(err.unwrap_or("")),
        Line::from(""),
        focus_line(0, focus, format!("Name: {}", draft.name)),
        focus_line(1, focus, format!("Enabled: {} (Space toggles)", draft.enabled)),
        focus_line(2, focus, format!("Severity: {} (v cycle)", sev)),
        focus_line(3, focus, format!("Cooldown sec: {}", draft.cooldown_seconds)),
        focus_line(4, focus, kind_line),
    ];

    match draft.condition_kind {
        ConditionKind::PortOpened | ConditionKind::PortClosed => {
            lines.push(focus_line(5, focus, format!("Port: {}", draft.port)));
        }
        ConditionKind::PortRangeActivity => {
            lines.push(focus_line(5, focus, format!("Start port: {}", draft.start_port)));
            lines.push(focus_line(6, focus, format!("End port: {}", draft.end_port)));
        }
        ConditionKind::ExternalConnection => {
            lines.push(focus_line(
                5,
                focus,
                format!("IP regex: {}", draft.ip_pattern),
            ));
            lines.push(focus_line(
                6,
                focus,
                format!("Exclude private: {} (Space)", draft.exclude_private),
            ));
        }
        ConditionKind::ProcessCpuThreshold => {
            lines.push(focus_line(
                5,
                focus,
                format!("Process regex: {}", draft.process_pattern),
            ));
            lines.push(focus_line(
                6,
                focus,
                format!("CPU % threshold: {}", draft.threshold_percent),
            ));
        }
        ConditionKind::ProcessMemoryThreshold => {
            lines.push(focus_line(
                5,
                focus,
                format!("Process regex: {}", draft.process_pattern),
            ));
            lines.push(focus_line(
                6,
                focus,
                format!("Memory MB threshold: {}", draft.threshold_mb),
            ));
        }
        ConditionKind::UnknownProcessListening => {}
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Tab/Shift+Tab field • Enter save • Esc cancel • type edits focused field",
        Style::default().fg(Color::DarkGray),
    )));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Edit rule ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn focus_line(idx: usize, focus: usize, s: String) -> Line<'static> {
    let style = if idx == focus {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Line::from(Span::styled(s, style))
}
