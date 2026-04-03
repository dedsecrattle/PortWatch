use crate::models::PortRecord;
use super::{Alert, AlertCondition, AlertRule};
use regex::Regex;
use std::net::IpAddr;

pub struct AlertEvaluator;

impl AlertEvaluator {
    pub fn evaluate_port_changes(
        rule: &AlertRule,
        previous_ports: &[PortRecord],
        current_ports: &[PortRecord],
    ) -> Option<Alert> {
        match &rule.condition {
            AlertCondition::PortOpened { port } => {
                let was_open = previous_ports.iter().any(|p| p.local_port == *port);
                let is_open = current_ports.iter().any(|p| p.local_port == *port);
                
                if !was_open && is_open {
                    let record = current_ports.iter().find(|p| p.local_port == *port)?;
                    return Some(Alert::new(
                        rule.id.clone(),
                        format!(
                            "Port {} opened by {}",
                            port,
                            record.process_name.as_deref().unwrap_or("unknown")
                        ),
                        rule.severity,
                    ));
                }
            }
            
            AlertCondition::PortClosed { port } => {
                let was_open = previous_ports.iter().any(|p| p.local_port == *port);
                let is_open = current_ports.iter().any(|p| p.local_port == *port);
                
                if was_open && !is_open {
                    return Some(Alert::new(
                        rule.id.clone(),
                        format!("Port {} closed", port),
                        rule.severity,
                    ));
                }
            }
            
            AlertCondition::PortRangeActivity { start_port, end_port } => {
                for port in current_ports {
                    if port.local_port >= *start_port && port.local_port <= *end_port {
                        let was_active = previous_ports.iter()
                            .any(|p| p.local_port == port.local_port);
                        
                        if !was_active {
                            return Some(Alert::new(
                                rule.id.clone(),
                                format!(
                                    "Activity detected on port {} (range {}-{})",
                                    port.local_port, start_port, end_port
                                ),
                                rule.severity,
                            ));
                        }
                    }
                }
            }
            
            _ => {}
        }
        None
    }

    pub fn evaluate_connections(
        rule: &AlertRule,
        ports: &[PortRecord],
    ) -> Option<Alert> {
        if let AlertCondition::ExternalConnection {
            ip_pattern,
            exclude_private,
        } = &rule.condition
        {
            let pattern = Regex::new(ip_pattern).ok()?;

            for port in ports {
                if let Some(ref remote_addr) = port.remote_addr {
                    if *exclude_private && Self::is_private_ip(remote_addr) {
                        continue;
                    }

                    if pattern.is_match(remote_addr) {
                        return Some(Alert::new(
                            rule.id.clone(),
                            format!(
                                "External connection to {} on port {}",
                                remote_addr, port.local_port
                            ),
                            rule.severity,
                        ));
                    }
                }
            }
        }
        None
    }

    pub fn evaluate_process_cpu(
        rule: &AlertRule,
        ports: &[PortRecord],
        cpu_usage: &std::collections::HashMap<u32, f32>,
    ) -> Option<Alert> {
        if let AlertCondition::ProcessCpuThreshold { process_pattern, threshold_percent } = &rule.condition {
            let pattern = Regex::new(process_pattern).ok()?;
            
            for port in ports {
                if let (Some(ref name), Some(pid)) = (&port.process_name, port.pid) {
                    if pattern.is_match(name) {
                        if let Some(&cpu) = cpu_usage.get(&pid) {
                            if cpu > *threshold_percent {
                                return Some(Alert::new(
                                    rule.id.clone(),
                                    format!(
                                        "Process {} (PID: {}) CPU usage: {:.1}% (threshold: {:.1}%)",
                                        name, pid, cpu, threshold_percent
                                    ),
                                    rule.severity,
                                ));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn evaluate_process_memory(
        rule: &AlertRule,
        ports: &[PortRecord],
        memory_usage: &std::collections::HashMap<u32, u64>,
    ) -> Option<Alert> {
        if let AlertCondition::ProcessMemoryThreshold { process_pattern, threshold_mb } = &rule.condition {
            let pattern = Regex::new(process_pattern).ok()?;
            let threshold_bytes = threshold_mb * 1024 * 1024;
            
            for port in ports {
                if let (Some(ref name), Some(pid)) = (&port.process_name, port.pid) {
                    if pattern.is_match(name) {
                        if let Some(&memory) = memory_usage.get(&pid) {
                            if memory > threshold_bytes {
                                return Some(Alert::new(
                                    rule.id.clone(),
                                    format!(
                                        "Process {} (PID: {}) memory usage: {} MB (threshold: {} MB)",
                                        name, pid, memory / 1024 / 1024, threshold_mb
                                    ),
                                    rule.severity,
                                ));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn is_private_ip(ip: &str) -> bool {
        if let Ok(addr) = ip.parse::<IpAddr>() {
            match addr {
                IpAddr::V4(ipv4) => {
                    let octets = ipv4.octets();
                    octets[0] == 127 ||
                    octets[0] == 10 ||
                    (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31) ||
                    (octets[0] == 192 && octets[1] == 168) ||
                    octets[0] == 0
                }
                IpAddr::V6(ipv6) => {
                    ipv6.is_loopback() || ipv6.is_unspecified()
                }
            }
        } else {
            false
        }
    }
}
