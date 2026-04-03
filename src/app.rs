use crate::alerts::{AlertEvaluator, AlertManager, Notifier};
use crate::backends::Backend;
use crate::events::Action;
use crate::models::{PortRecord, ProcessDetails};
use anyhow::Result;
use std::collections::HashMap;

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
    backend: Backend,
    previous_ports: Vec<PortRecord>,
    cpu_usage: HashMap<u32, f32>,
    memory_usage: HashMap<u32, u64>,
}

impl AppState {
    pub fn new() -> Self {
        let mut alert_manager = AlertManager::new();
        
        // Load alert configuration or use defaults
        let config = crate::alerts::AlertConfig::load().unwrap_or_default();
        
        for rule in config.rules {
            alert_manager.add_rule(rule);
        }
        
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
            backend: Backend::new(),
            previous_ports: Vec::new(),
            cpu_usage: HashMap::new(),
            memory_usage: HashMap::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        match self.backend.scan_ports() {
            Ok(ports) => {
                // Evaluate alerts before updating ports
                self.evaluate_alerts(&ports);
                
                // Update previous ports for next comparison
                self.previous_ports = self.ports.clone();
                self.ports = ports;
                
                self.status_message = Some(format!("Refreshed: {} connections found", self.ports.len()));
                
                if self.selected_index >= self.filtered_ports().len() && !self.filtered_ports().is_empty() {
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
        // Collect enabled rules to avoid borrow checker issues
        let enabled_rules: Vec<_> = self.alert_manager.get_enabled_rules()
            .into_iter()
            .cloned()
            .collect();
        
        // Only update process metrics if we have CPU/Memory threshold rules
        let needs_metrics = enabled_rules.iter().any(|rule| {
            matches!(
                rule.condition,
                crate::alerts::AlertCondition::ProcessCpuThreshold { .. } |
                crate::alerts::AlertCondition::ProcessMemoryThreshold { .. }
            )
        });
        
        if needs_metrics {
            self.update_process_metrics(current_ports);
        }
        
        for rule in enabled_rules {
            if !self.alert_manager.can_trigger(&rule.id, rule.cooldown_seconds) {
                continue;
            }

            let alert = match &rule.condition {
                crate::alerts::AlertCondition::PortOpened { .. } |
                crate::alerts::AlertCondition::PortClosed { .. } |
                crate::alerts::AlertCondition::PortRangeActivity { .. } => {
                    AlertEvaluator::evaluate_port_changes(&rule, &self.previous_ports, current_ports)
                }
                crate::alerts::AlertCondition::ExternalConnection { .. } => {
                    AlertEvaluator::evaluate_connections(&rule, current_ports)
                }
                crate::alerts::AlertCondition::ProcessCpuThreshold { .. } => {
                    AlertEvaluator::evaluate_process_cpu(&rule, current_ports, &self.cpu_usage)
                }
                crate::alerts::AlertCondition::ProcessMemoryThreshold { .. } => {
                    AlertEvaluator::evaluate_process_memory(&rule, current_ports, &self.memory_usage)
                }
                _ => None,
            };

            if let Some(alert) = alert {
                self.status_message = Some(format!("Alert: {}", alert.message));
                let _ = self.notifier.send(&alert);
                self.alert_manager.trigger_alert(alert);
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
                            self.status_message = Some(format!("Process {} {} (PID: {})", 
                                process_name.as_deref().unwrap_or("unknown"), 
                                action_type, 
                                pid));
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
