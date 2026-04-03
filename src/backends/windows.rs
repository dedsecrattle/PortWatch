use crate::backends::{PortBackend, ProcessBackend};
use crate::models::{ConnectionState, PortRecord, ProcessDetails, Protocol};
use anyhow::{Context, Result};
use std::process::Command;
use sysinfo::{ProcessesToUpdate, System};

pub struct WindowsPortBackend {
    system: System,
}

impl WindowsPortBackend {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    fn parse_netstat_output(&mut self, output: &str) -> Vec<PortRecord> {
        let mut records = Vec::new();
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        for line in output.lines().skip(4) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }

            let protocol = match parts[0] {
                "TCP" => Protocol::Tcp,
                "UDP" => Protocol::Udp,
                _ => continue,
            };

            let local = parts[1];
            let (local_addr, local_port) = Self::parse_address(local);

            let (remote_addr, remote_port, state, pid_idx) = if protocol == Protocol::Tcp {
                let remote = parts[2];
                let (r_addr, r_port) = Self::parse_address(remote);
                let state = Self::parse_state(parts[3]);
                let pid_idx = 4;
                (Some(r_addr), Some(r_port), state, pid_idx)
            } else {
                (None, None, ConnectionState::Listen, 3)
            };

            let pid = if parts.len() > pid_idx {
                parts[pid_idx].parse::<u32>().ok()
            } else {
                None
            };

            let (process_name, exe, cmdline, user) = if let Some(p) = pid {
                if let Some(process) = self.system.process(sysinfo::Pid::from_u32(p)) {
                    let cmd: Vec<String> = process.cmd().iter()
                        .filter_map(|s| s.to_str().map(|s| s.to_string()))
                        .collect();
                    (
                        Some(process.name().to_str().unwrap_or("unknown").to_string()),
                        process.exe().map(|p| p.to_path_buf()),
                        cmd,
                        process.user_id().map(|u| u.to_string()),
                    )
                } else {
                    (None, None, Vec::new(), None)
                }
            } else {
                (None, None, Vec::new(), None)
            };

            records.push(PortRecord {
                protocol,
                local_addr,
                local_port,
                remote_addr,
                remote_port,
                state,
                pid,
                process_name,
                exe,
                cmdline,
                user,
            });
        }

        records
    }

    fn parse_address(addr: &str) -> (String, u16) {
        if let Some(colon_pos) = addr.rfind(':') {
            let ip = addr[..colon_pos].to_string();
            let port = addr[colon_pos + 1..].parse::<u16>().unwrap_or(0);
            
            let ip = if ip.starts_with('[') && ip.ends_with(']') {
                ip[1..ip.len()-1].to_string()
            } else {
                ip
            };
            
            (ip, port)
        } else {
            ("0.0.0.0".to_string(), 0)
        }
    }

    fn parse_state(state: &str) -> ConnectionState {
        match state {
            "LISTENING" => ConnectionState::Listen,
            "ESTABLISHED" => ConnectionState::Established,
            "SYN_SENT" => ConnectionState::SynSent,
            "SYN_RECEIVED" => ConnectionState::SynRecv,
            "FIN_WAIT_1" => ConnectionState::FinWait1,
            "FIN_WAIT_2" => ConnectionState::FinWait2,
            "TIME_WAIT" => ConnectionState::TimeWait,
            "CLOSE_WAIT" => ConnectionState::CloseWait,
            "LAST_ACK" => ConnectionState::LastAck,
            "CLOSING" => ConnectionState::Closing,
            "CLOSED" => ConnectionState::Close,
            _ => ConnectionState::Unknown,
        }
    }
}

impl PortBackend for WindowsPortBackend {
    fn scan_ports(&mut self) -> Result<Vec<PortRecord>> {
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .context("Failed to execute netstat")?;

        if !output.status.success() {
            anyhow::bail!("netstat command failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(self.parse_netstat_output(&stdout))
    }
}

pub struct WindowsProcessBackend {
    system: System,
}

impl WindowsProcessBackend {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

impl ProcessBackend for WindowsProcessBackend {
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
        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(pid)) {
            if graceful {
                process.kill();
            } else {
                process.kill();
            }
            Ok(())
        } else {
            anyhow::bail!("Process not found")
        }
    }
}
