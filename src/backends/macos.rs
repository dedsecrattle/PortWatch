use crate::backends::{PortBackend, ProcessBackend};
use crate::models::{ConnectionState, PortRecord, ProcessDetails, Protocol};
use anyhow::{Context, Result};
use std::process::Command;
use sysinfo::{ProcessesToUpdate, System};

pub struct MacOsPortBackend {
    system: System,
}

impl MacOsPortBackend {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    fn parse_lsof_output(&mut self, output: &str) -> Vec<PortRecord> {
        let mut records = Vec::new();
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }

            let process_name = parts[0].to_string();
            let pid = parts[1].parse::<u32>().ok();
            let protocol_str = parts[7];
            let name_field = parts[8];

            let protocol = if protocol_str.contains("TCP") {
                Protocol::Tcp
            } else if protocol_str.contains("UDP") {
                Protocol::Udp
            } else {
                continue;
            };

            // Extract state from the last field (which may contain state in parentheses)
            let state = if let Some(last_part) = parts.last() {
                if last_part.contains("(LISTEN)") {
                    ConnectionState::Listen
                } else if last_part.contains("(ESTABLISHED)") {
                    ConnectionState::Established
                } else if last_part.contains("(CLOSE_WAIT)") {
                    ConnectionState::CloseWait
                } else if last_part.contains("(TIME_WAIT)") {
                    ConnectionState::TimeWait
                } else if last_part.contains("(SYN_SENT)") {
                    ConnectionState::SynSent
                } else if last_part.contains("(SYN_RCVD)") {
                    ConnectionState::SynRecv
                } else if last_part.contains("(FIN_WAIT_1)") {
                    ConnectionState::FinWait1
                } else if last_part.contains("(FIN_WAIT_2)") {
                    ConnectionState::FinWait2
                } else if last_part.contains("(CLOSING)") {
                    ConnectionState::Closing
                } else if last_part.contains("(LAST_ACK)") {
                    ConnectionState::LastAck
                } else if last_part.contains("(CLOSED)") {
                    ConnectionState::Close
                } else if protocol == Protocol::Udp {
                    // UDP doesn't have states, treat as Listen
                    ConnectionState::Listen
                } else {
                    ConnectionState::Unknown
                }
            } else {
                ConnectionState::Unknown
            };

            let (local_addr, local_port, remote_addr, remote_port) = 
                Self::parse_address_field(name_field);

            let (exe, cmdline, user) = if let Some(p) = pid {
                if let Some(process) = self.system.process(sysinfo::Pid::from_u32(p)) {
                    let cmd: Vec<String> = process.cmd().iter()
                        .filter_map(|s| s.to_str().map(|s| s.to_string()))
                        .collect();
                    (
                        process.exe().map(|p| p.to_path_buf()),
                        cmd,
                        process.user_id().map(|u| u.to_string()),
                    )
                } else {
                    (None, Vec::new(), None)
                }
            } else {
                (None, Vec::new(), None)
            };

            records.push(PortRecord {
                protocol,
                local_addr,
                local_port,
                remote_addr,
                remote_port,
                state,
                pid,
                process_name: Some(process_name),
                exe,
                cmdline,
                user,
            });
        }

        records
    }

    fn parse_address_field(field: &str) -> (String, u16, Option<String>, Option<u16>) {
        if field.contains("->") {
            let parts: Vec<&str> = field.split("->").collect();
            if parts.len() == 2 {
                let (local_addr, local_port) = Self::parse_single_address(parts[0]);
                let (remote_addr, remote_port) = Self::parse_single_address(parts[1]);
                return (local_addr, local_port, Some(remote_addr), Some(remote_port));
            }
        }

        let (addr, port) = Self::parse_single_address(field);
        (addr, port, None, None)
    }

    fn parse_single_address(addr: &str) -> (String, u16) {
        if let Some(colon_pos) = addr.rfind(':') {
            let ip = addr[..colon_pos].to_string();
            let port = addr[colon_pos + 1..].parse::<u16>().unwrap_or(0);
            
            let ip = if ip.starts_with('[') && ip.ends_with(']') {
                ip[1..ip.len()-1].to_string()
            } else if ip == "*" {
                "0.0.0.0".to_string()
            } else {
                ip
            };
            
            (ip, port)
        } else {
            ("0.0.0.0".to_string(), 0)
        }
    }
}

impl PortBackend for MacOsPortBackend {
    fn scan_ports(&mut self) -> Result<Vec<PortRecord>> {
        let output = Command::new("lsof")
            .args(["-i", "-n", "-P"])
            .output()
            .context("Failed to execute lsof")?;

        if !output.status.success() {
            anyhow::bail!("lsof command failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut records = self.parse_lsof_output(&stdout);
        
        // Deduplicate: only remove exact duplicates (same PID, port, protocol, state, and address)
        // Different PIDs on the same port are legitimate (e.g., parent/worker processes)
        let mut seen = std::collections::HashSet::new();
        records.retain(|record| {
            let key = (
                record.protocol as u8,
                record.local_addr.clone(),
                record.local_port,
                record.pid,
                record.state as u8,
            );
            seen.insert(key)
        });
        
        Ok(records)
    }
}

pub struct MacOsProcessBackend {
    system: System,
}

impl MacOsProcessBackend {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

impl ProcessBackend for MacOsProcessBackend {
    fn process_details(&mut self, pid: u32) -> Result<ProcessDetails> {
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let process = self
            .system
            .process(sysinfo::Pid::from_u32(pid))
            .context("Process not found")?;

        let env_preview: Vec<(String, String)> = process
            .environ()
            .iter()
            .take(10)
            .filter_map(|e| {
                let e_str = e.to_str()?;
                let parts: Vec<&str> = e_str.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();

        let cmdline: Vec<String> = process.cmd().iter()
            .filter_map(|s| s.to_str().map(|s| s.to_string()))
            .collect();

        Ok(ProcessDetails {
            pid,
            parent_pid: process.parent().map(|p| p.as_u32()),
            name: process.name().to_str().unwrap_or("unknown").to_string(),
            exe: process.exe().map(|p| p.to_path_buf()),
            cwd: process.cwd().map(|p| p.to_path_buf()),
            memory_bytes: process.memory(),
            cpu_percent: process.cpu_usage(),
            start_time: Some(process.start_time()),
            cmdline,
            env_preview,
            user: process.user_id().map(|u| u.to_string()),
        })
    }

    fn stop_process(&mut self, pid: u32, graceful: bool) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let signal = if graceful {
            Signal::SIGTERM
        } else {
            Signal::SIGKILL
        };

        kill(Pid::from_raw(pid as i32), signal)
            .context("Failed to send signal to process")?;

        Ok(())
    }
}
