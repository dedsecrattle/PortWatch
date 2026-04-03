use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub normal: Style,
    pub selected: Style,
    pub header: Style,
    pub border: Style,
    pub border_focused: Style,
    pub title: Style,
    pub listen: Style,
    pub established: Style,
    pub other_state: Style,
    pub tcp: Style,
    pub udp: Style,
    pub port: Style,
    pub pid: Style,
    pub process_name: Style,
    pub address: Style,
    pub label: Style,
    pub value: Style,
    pub value_highlight: Style,
    pub status_bar: Style,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            normal: Style::default().fg(Color::Rgb(200, 200, 200)),
            selected: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(0, 255, 255))
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            border: Style::default().fg(Color::Rgb(100, 100, 100)),
            border_focused: Style::default()
                .fg(Color::Rgb(0, 255, 255))
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::Rgb(138, 43, 226))
                .add_modifier(Modifier::BOLD),
            listen: Style::default()
                .fg(Color::Rgb(50, 205, 50))
                .add_modifier(Modifier::BOLD),
            established: Style::default()
                .fg(Color::Rgb(0, 191, 255))
                .add_modifier(Modifier::BOLD),
            other_state: Style::default()
                .fg(Color::Rgb(255, 165, 0)),
            tcp: Style::default()
                .fg(Color::Rgb(135, 206, 250)),
            udp: Style::default()
                .fg(Color::Rgb(255, 182, 193)),
            port: Style::default()
                .fg(Color::Rgb(255, 255, 102))
                .add_modifier(Modifier::BOLD),
            pid: Style::default()
                .fg(Color::Rgb(173, 216, 230)),
            process_name: Style::default()
                .fg(Color::Rgb(255, 192, 203))
                .add_modifier(Modifier::BOLD),
            address: Style::default()
                .fg(Color::Rgb(152, 251, 152)),
            label: Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD),
            value: Style::default()
                .fg(Color::Rgb(173, 216, 230)),
            value_highlight: Style::default()
                .fg(Color::Rgb(255, 105, 180))
                .add_modifier(Modifier::BOLD),
            status_bar: Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(60, 60, 60)),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}
