use crate::backends::Backend;
use crate::events::Action;
use crate::models::{PortRecord, ProcessDetails};
use anyhow::Result;

pub struct AppState {
    pub ports: Vec<PortRecord>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub status_message: Option<String>,
    pub process_details: Option<ProcessDetails>,
    pub show_help: bool,
    backend: Backend,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ports: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            filter: String::new(),
            status_message: None,
            process_details: None,
            show_help: false,
            backend: Backend::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        match self.backend.scan_ports() {
            Ok(ports) => {
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
