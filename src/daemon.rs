use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::alerts::{run_alert_cycle, AlertConfig, AlertManager, Notifier};
use crate::backends::Backend;
use crate::models::PortRecord;

pub struct DaemonState {
    pub alert_manager: AlertManager,
    notifier: Notifier,
    backend: Backend,
    previous_ports: Vec<PortRecord>,
    cpu_usage: HashMap<u32, f32>,
    memory_usage: HashMap<u32, u64>,
    reload_config_each_tick: bool,
}

impl DaemonState {
    pub fn new(reload_config_each_tick: bool) -> Self {
        let mut alert_manager = AlertManager::new();
        let (config, _): (AlertConfig, _) =
            AlertConfig::load().unwrap_or_else(|_| (Default::default(), None));
        alert_manager.set_rules(config.rules);

        Self {
            alert_manager,
            notifier: Notifier::new(true),
            backend: Backend::new(),
            previous_ports: Vec::new(),
            cpu_usage: HashMap::new(),
            memory_usage: HashMap::new(),
            reload_config_each_tick,
        }
    }

    fn reload_rules_from_disk(&mut self) {
        match AlertConfig::load() {
            Ok((config, _)) => {
                self.alert_manager.set_rules(config.rules);
            }
            Err(e) => {
                eprintln!("[portwatch daemon] warning: could not reload alerts config: {}", e);
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

    pub fn tick(&mut self) -> Result<()> {
        if self.reload_config_each_tick {
            self.reload_rules_from_disk();
        }

        let ports = self.backend.scan_ports()?;

        let needs_metrics = self.alert_manager.get_enabled_rules().iter().any(|rule| {
            matches!(
                rule.condition,
                crate::alerts::AlertCondition::ProcessCpuThreshold { .. }
                    | crate::alerts::AlertCondition::ProcessMemoryThreshold { .. }
            )
        });

        if needs_metrics {
            self.update_process_metrics(&ports);
        }

        let triggered = run_alert_cycle(
            &mut self.alert_manager,
            &self.notifier,
            &self.previous_ports,
            &ports,
            &self.cpu_usage,
            &self.memory_usage,
        );

        for t in &triggered {
            eprintln!("[portwatch] {}", t.message);
            if let Err(e) = &t.notify_result {
                eprintln!("[portwatch] notification failed: {}", e);
            }
        }

        self.previous_ports = ports;
        Ok(())
    }
}

pub fn run_daemon_loop(interval: Duration, reload_config_each_tick: bool) -> Result<()> {
    eprintln!(
        "[portwatch daemon] started (interval: {:?}{})",
        interval,
        if reload_config_each_tick {
            ", reloading config each tick"
        } else {
            ""
        }
    );

    let mut state = DaemonState::new(reload_config_each_tick);
    let mut last = Instant::now();

    loop {
        if let Err(e) = state.tick() {
            eprintln!("[portwatch daemon] scan error: {}", e);
        }

        let elapsed = last.elapsed();
        if elapsed < interval {
            std::thread::sleep(interval - elapsed);
        }
        last = Instant::now();
    }
}
