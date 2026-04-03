use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub normal: Style,
    pub selected: Style,
    pub header: Style,
    pub border: Style,
    pub listen: Style,
    pub established: Style,
    pub other_state: Style,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            normal: Style::default().fg(Color::White),
            selected: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::Gray),
            listen: Style::default().fg(Color::Green),
            established: Style::default().fg(Color::Cyan),
            other_state: Style::default().fg(Color::Yellow),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}
