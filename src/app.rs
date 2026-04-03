use crate::alerts::{run_alert_cycle, AlertManager, Notifier};
use crate::backends::Backend;
use crate::events::Action;
use crate::models::{PortRecord, ProcessDetails};
use crate::ui::alert_rules::{AlertRuleDraft, AlertRulesPanel};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AppState {
    pub ports: Vec<PortRecord>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub status_message: Option<String>,
    pub process_details: Option<ProcessDetails>,
    pub show_help: bool,
    pub alert_manager: AlertManager,
    pub notifier: Notifier,
    pub show_alerts: bool,
    pub alert_rules: AlertRulesPanel,
    backend: Backend,
    previous_ports: Vec<PortRecord>,
    cpu_usage: HashMap<u32, f32>,
    memory_usage: HashMap<u32, u64>,
}

impl AppState {
    pub fn new() -> Self {
        let mut alert_manager = AlertManager::new();

        let (config, _): (crate::alerts::AlertConfig, Option<PathBuf>) =
            crate::alerts::AlertConfig::load().unwrap_or_else(|_| (Default::default(), None));
        alert_manager.set_rules(config.rules);

        Self {
            ports: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            filter: String::new(),
            status_message: None,
            process_details: None,
            show_help: false,
            alert_manager,
            notifier: Notifier::new(true),
            show_alerts: false,
            alert_rules: AlertRulesPanel::default(),
            backend: Backend::new(),
            previous_ports: Vec::new(),
            cpu_usage: HashMap::new(),
            memory_usage: HashMap::new(),
        }
    }

    fn persist_alert_rules(&mut self) {
        let rules = self.alert_manager.get_rules().to_vec();
        let cfg = crate::alerts::AlertConfig { rules };
        match cfg.save() {
            Ok(path) => {
                self.status_message = Some(format!("Saved alert rules to {}", path.display()));
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to save alert rules: {}", e));
            }
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        match self.backend.scan_ports() {
            Ok(ports) => {
                self.evaluate_alerts(&ports);

                self.previous_ports = self.ports.clone();
                self.ports = ports;

                self.status_message =
                    Some(format!("Refreshed: {} connections found", self.ports.len()));

                if self.selected_index >= self.filtered_ports().len()
                    && !self.filtered_ports().is_empty()
                {
                    self.selected_index = self.filtered_ports().len() - 1;
                }

                if let Some(selected) = self.get_selected_port() {
                    if let Some(pid) = selected.pid {
                        self.load_process_details(pid);
                    }
                }

                Ok(())
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
                Err(e)
            }
        }
    }

    fn evaluate_alerts(&mut self, current_ports: &[PortRecord]) {
        let needs_metrics = self.alert_manager.get_enabled_rules().iter().any(|rule| {
            matches!(
                rule.condition,
                crate::alerts::AlertCondition::ProcessCpuThreshold { .. }
                    | crate::alerts::AlertCondition::ProcessMemoryThreshold { .. }
            )
        });

        if needs_metrics {
            self.update_process_metrics(current_ports);
        }

        let triggered = run_alert_cycle(
            &mut self.alert_manager,
            &self.notifier,
            &self.previous_ports,
            current_ports,
            &self.cpu_usage,
            &self.memory_usage,
        );

        if let Some(last) = triggered.last() {
            self.status_message = Some(format!("Alert: {}", last.message));
            if let Err(e) = &last.notify_result {
                self.status_message = Some(format!(
                    "Alert: {} (desktop notify failed: {})",
                    last.message, e
                ));
            }
        }
    }

    fn update_process_metrics(&mut self, ports: &[PortRecord]) {
        self.cpu_usage.clear();
        self.memory_usage.clear();

        for port in ports {
            if let Some(pid) = port.pid {
                if let Ok(details) = self.backend.process_details(pid) {
                    self.cpu_usage.insert(pid, details.cpu_percent);
                    self.memory_usage.insert(pid, details.memory_bytes);
                }
            }
        }
    }

    pub fn apply_action(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Quit => return Ok(true),

            Action::Refresh => {
                self.refresh()?;
            }

            Action::NavigateUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if let Some(selected) = self.get_selected_port() {
                        if let Some(pid) = selected.pid {
                            self.load_process_details(pid);
                        }
                    }
                }
            }

            Action::NavigateDown => {
                let filtered_len = self.filtered_ports().len();
                if filtered_len > 0 && self.selected_index < filtered_len - 1 {
                    self.selected_index += 1;
                    if let Some(selected) = self.get_selected_port() {
                        if let Some(pid) = selected.pid {
                            self.load_process_details(pid);
                        }
                    }
                }
            }

            Action::SelectItem => {
                if let Some(selected) = self.get_selected_port() {
                    if let Some(pid) = selected.pid {
                        self.load_process_details(pid);
                    }
                }
            }

            Action::KillProcess(graceful) => {
                let (pid_opt, process_name) = if let Some(selected) = self.get_selected_port() {
                    (selected.pid, selected.process_name.clone())
                } else {
                    (None, None)
                };

                if let Some(pid) = pid_opt {
                    match self.backend.stop_process(pid, graceful) {
                        Ok(_) => {
                            let action_type = if graceful { "terminated" } else { "killed" };
                            self.status_message = Some(format!(
                                "Process {} {} (PID: {})",
                                process_name.as_deref().unwrap_or("unknown"),
                                action_type,
                                pid
                            ));
                            self.refresh()?;
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to kill process: {}", e));
                        }
                    }
                } else {
                    self.status_message = Some("No process associated with this port".to_string());
                }
            }

            Action::StartFilter => {
                self.filter.clear();
            }

            Action::UpdateFilter(s) => {
                if s == "\x08" {
                    self.filter.pop();
                } else {
                    self.filter.push_str(&s);
                }
                self.selected_index = 0;
                self.scroll_offset = 0;
            }

            Action::ClearFilter => {
                self.filter.clear();
                self.selected_index = 0;
                self.scroll_offset = 0;
            }

            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }

            Action::ToggleAlerts => {
                self.show_alerts = !self.show_alerts;
            }

            Action::ToggleAlertRulesEditor => {
                self.alert_rules.open = !self.alert_rules.open;
                if !self.alert_rules.open {
                    self.alert_rules.form = None;
                    self.alert_rules.error = None;
                } else {
                    let n = self.alert_manager.get_rules().len();
                    if n > 0 && self.alert_rules.selected >= n {
                        self.alert_rules.selected = n - 1;
                    }
                }
            }

            Action::AlertRulesNavigateUp => {
                if self.alert_rules.selected > 0 {
                    self.alert_rules.selected -= 1;
                }
            }

            Action::AlertRulesNavigateDown => {
                let n = self.alert_manager.get_rules().len();
                if n > 0 && self.alert_rules.selected < n - 1 {
                    self.alert_rules.selected += 1;
                }
            }

            Action::AlertRulesNew => {
                self.alert_rules.form = Some(AlertRuleDraft::default());
                self.alert_rules.form_focus = 0;
                self.alert_rules.error = None;
            }

            Action::AlertRulesDelete => {
                let n = self.alert_manager.get_rules().len();
                if n == 0 {
                    return Ok(false);
                }
                let i = self.alert_rules.selected.min(n - 1);
                if self.alert_manager.delete_rule(i).is_ok() {
                    self.persist_alert_rules();
                    let new_len = self.alert_manager.get_rules().len();
                    if self.alert_rules.selected >= new_len {
                        self.alert_rules.selected = new_len.saturating_sub(1);
                    }
                }
            }

            Action::AlertRulesToggleEnabled => {
                let n = self.alert_manager.get_rules().len();
                if n == 0 {
                    return Ok(false);
                }
                let i = self.alert_rules.selected.min(n - 1);
                if self.alert_manager.toggle_rule_enabled(i).is_ok() {
                    self.persist_alert_rules();
                }
            }

            Action::AlertRulesEdit => {
                let rules = self.alert_manager.get_rules();
                if rules.is_empty() {
                    self.status_message = Some("No rules to edit. Press n to add one.".into());
                    return Ok(false);
                }
                let i = self.alert_rules.selected.min(rules.len() - 1);
                self.alert_rules.form = Some(AlertRuleDraft::from_rule(&rules[i], i));
                self.alert_rules.form_focus = 0;
                self.alert_rules.error = None;
            }

            Action::AlertRulesFormCancel => {
                self.alert_rules.form = None;
                self.alert_rules.error = None;
            }

            Action::AlertRulesFormSave => {
                if let Some(draft) = self.alert_rules.form.take() {
                    let idx = draft.editing_index;
                    match draft.to_rule() {
                        Ok(rule) => {
                            if let Some(i) = idx {
                                let _ = self.alert_manager.update_rule(i, rule);
                            } else {
                                self.alert_manager.add_rule(rule);
                            }
                            self.persist_alert_rules();
                            self.alert_rules.form = None;
                            self.alert_rules.error = None;
                        }
                        Err(e) => {
                            self.alert_rules.error = Some(e);
                            self.alert_rules.form = Some(draft);
                        }
                    }
                }
            }

            Action::AlertRulesFormNextField => {
                if let Some(ref d) = self.alert_rules.form {
                    let m = d.max_focus();
                    self.alert_rules.form_focus = if self.alert_rules.form_focus >= m {
                        0
                    } else {
                        self.alert_rules.form_focus + 1
                    };
                }
            }

            Action::AlertRulesFormPrevField => {
                if let Some(ref d) = self.alert_rules.form {
                    let m = d.max_focus();
                    self.alert_rules.form_focus = if self.alert_rules.form_focus == 0 {
                        m
                    } else {
                        self.alert_rules.form_focus - 1
                    };
                }
            }

            Action::AlertRulesFormToggleBool => {
                if let Some(ref mut d) = self.alert_rules.form {
                    d.toggle_bool_field(self.alert_rules.form_focus);
                }
            }

            Action::AlertRulesFormCycleSeverity => {
                if let Some(ref mut d) = self.alert_rules.form {
                    if self.alert_rules.form_focus == 2 {
                        d.cycle_severity();
                    }
                }
            }

            Action::AlertRulesFormSetConditionKind(n) => {
                if let Some(ref mut d) = self.alert_rules.form {
                    d.set_condition_kind(n);
                    self.alert_rules.form_focus = self.alert_rules.form_focus.min(d.max_focus());
                }
            }

            Action::AlertRulesFormInput(c) => {
                if let Some(ref mut d) = self.alert_rules.form {
                    d.append_to_field(self.alert_rules.form_focus, c);
                    self.alert_rules.error = None;
                }
            }

            Action::AlertRulesFormBackspace => {
                if let Some(ref mut d) = self.alert_rules.form {
                    d.backspace_field(self.alert_rules.form_focus);
                }
            }

            Action::None => {}
        }

        Ok(false)
    }

    pub fn filtered_ports(&self) -> Vec<&PortRecord> {
        self.ports
            .iter()
            .filter(|p| p.matches_filter(&self.filter))
            .collect()
    }

    pub fn get_selected_port(&self) -> Option<&PortRecord> {
        let filtered = self.filtered_ports();
        filtered.get(self.selected_index).copied()
    }

    fn load_process_details(&mut self, pid: u32) {
        match self.backend.process_details(pid) {
            Ok(details) => {
                self.process_details = Some(details);
            }
            Err(_) => {
                self.process_details = None;
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
