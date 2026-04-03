use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Refresh,
    NavigateUp,
    NavigateDown,
    SelectItem,
    KillProcess(bool),
    StartFilter,
    UpdateFilter(String),
    ClearFilter,
    ToggleHelp,
    None,
}

pub struct EventHandler {
    filter_mode: bool,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            filter_mode: false,
        }
    }

    pub fn is_filter_mode(&self) -> bool {
        self.filter_mode
    }

    pub fn next_action(&mut self, timeout: Duration) -> anyhow::Result<Action> {
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                return Ok(self.handle_key(key));
            }
        }
        Ok(Action::None)
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if self.filter_mode {
            return self.handle_filter_key(key);
        }

        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Char('r') => Action::Refresh,
            KeyCode::Char('k') => Action::KillProcess(true),
            KeyCode::Char('K') => Action::KillProcess(false),
            KeyCode::Up => Action::NavigateUp,
            KeyCode::Down => Action::NavigateDown,
            KeyCode::Enter => Action::SelectItem,
            KeyCode::Char('/') => {
                self.filter_mode = true;
                Action::StartFilter
            }
            KeyCode::Esc => Action::ClearFilter,
            KeyCode::Char('?') => Action::ToggleHelp,
            _ => Action::None,
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.filter_mode = false;
                Action::ClearFilter
            }
            KeyCode::Enter => {
                self.filter_mode = false;
                Action::None
            }
            KeyCode::Backspace => Action::UpdateFilter(String::from("\x08")),
            KeyCode::Char(c) => Action::UpdateFilter(c.to_string()),
            _ => Action::None,
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
