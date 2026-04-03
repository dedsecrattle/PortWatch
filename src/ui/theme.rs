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
    pub footer_text: Style,
    pub footer_key: Style,
}

impl Theme {
    pub fn new() -> Self {
        // Use terminal's default colors for better light/dark mode compatibility
        // These ANSI colors adapt to the terminal's color scheme
        Self {
            normal: Style::default(),
            selected: Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            border: Style::default().fg(Color::DarkGray),
            border_focused: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            listen: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            established: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            other_state: Style::default()
                .fg(Color::Yellow),
            tcp: Style::default()
                .fg(Color::Cyan),
            udp: Style::default()
                .fg(Color::LightMagenta),
            port: Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
            pid: Style::default()
                .fg(Color::LightBlue),
            process_name: Style::default()
                .fg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD),
            address: Style::default()
                .fg(Color::LightGreen),
            label: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            value: Style::default()
                .fg(Color::LightCyan),
            value_highlight: Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
            status_bar: Style::default()
                .bg(Color::Rgb(0, 0, 0)),
            footer_text: Style::default()
                .fg(Color::White),
            footer_key: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}
