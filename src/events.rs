use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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
    ToggleAlerts,
    ToggleAlertRulesEditor,
    AlertRulesNavigateUp,
    AlertRulesNavigateDown,
    AlertRulesNew,
    AlertRulesDelete,
    AlertRulesToggleEnabled,
    AlertRulesEdit,
    AlertRulesFormCancel,
    AlertRulesFormSave,
    AlertRulesFormNextField,
    AlertRulesFormPrevField,
    AlertRulesFormToggleBool,
    AlertRulesFormCycleSeverity,
    AlertRulesFormSetConditionKind(u8),
    AlertRulesFormInput(char),
    AlertRulesFormBackspace,
    None,
}

#[derive(Clone, Copy, Default)]
pub struct InputContext {
    pub show_help: bool,
    pub alert_rules_open: bool,
    pub alert_rule_form: bool,
    pub alert_rule_form_focus: usize,
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

    pub fn next_action(
        &mut self,
        timeout: Duration,
        ctx: InputContext,
    ) -> anyhow::Result<Action> {
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    return Ok(self.handle_key(key, ctx));
                }
            }
        }
        Ok(Action::None)
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: InputContext) -> Action {
        if self.filter_mode {
            return self.handle_filter_key(key);
        }

        if ctx.show_help {
            return match key.code {
                KeyCode::Esc | KeyCode::Char('?') => Action::ToggleHelp,
                _ => Action::None,
            };
        }

        if ctx.alert_rule_form {
            return self.handle_alert_rule_form_key(key, ctx.alert_rule_form_focus);
        }

        if ctx.alert_rules_open {
            return self.handle_alert_rules_list_key(key);
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
            KeyCode::Char('a') => Action::ToggleAlerts,
            KeyCode::Char('E') => Action::ToggleAlertRulesEditor,
            _ => Action::None,
        }
    }

    fn handle_alert_rules_list_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc | KeyCode::Char('E') => Action::ToggleAlertRulesEditor,
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Up => Action::AlertRulesNavigateUp,
            KeyCode::Down => Action::AlertRulesNavigateDown,
            KeyCode::Char('n') => Action::AlertRulesNew,
            KeyCode::Char('d') => Action::AlertRulesDelete,
            KeyCode::Char('t') => Action::AlertRulesToggleEnabled,
            KeyCode::Char('r') => Action::Refresh,
            KeyCode::Enter => Action::AlertRulesEdit,
            _ => Action::None,
        }
    }

    fn handle_alert_rule_form_key(&mut self, key: KeyEvent, focus: usize) -> Action {
        match key.code {
            KeyCode::Esc => Action::AlertRulesFormCancel,
            KeyCode::Enter => Action::AlertRulesFormSave,
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    Action::AlertRulesFormPrevField
                } else {
                    Action::AlertRulesFormNextField
                }
            }
            KeyCode::Backspace => Action::AlertRulesFormBackspace,
            KeyCode::Char(' ') => Action::AlertRulesFormToggleBool,
            KeyCode::Char('v') | KeyCode::Char('V') => Action::AlertRulesFormCycleSeverity,
            KeyCode::Char(c @ '0'..='6') => {
                if focus == 4 {
                    Action::AlertRulesFormSetConditionKind(c as u8 - b'0')
                } else {
                    Action::AlertRulesFormInput(c)
                }
            }
            KeyCode::Char(c) => Action::AlertRulesFormInput(c),
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
