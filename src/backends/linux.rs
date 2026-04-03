use crate::backends::{PortBackend, ProcessBackend};
use crate::models::{ConnectionState, PortRecord, ProcessDetails, Protocol};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use sysinfo::{ProcessesToUpdate, System};

pub struct LinuxPortBackend {
    system: System,
}

impl LinuxPortBackend {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    fn parse_tcp_state(state: &str) -> ConnectionState {
        match state {
            "01" => ConnectionState::Established,
            "02" => ConnectionState::SynSent,
            "03" => ConnectionState::SynRecv,
            "04" => ConnectionState::FinWait1,
            "05" => ConnectionState::FinWait2,
            "06" => ConnectionState::TimeWait,
            "07" => ConnectionState::Close,
            "08" => ConnectionState::CloseWait,
            "09" => ConnectionState::LastAck,
            "0A" => ConnectionState::Listen,
            "0B" => ConnectionState::Closing,
            _ => ConnectionState::Unknown,
        }
    }

    fn parse_hex_addr(hex: &str) -> Result<(String, u16)> {
        let parts: Vec<&str> = hex.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid address format");
        }

        let addr_hex = parts[0];
        let port_hex = parts[1];

        let port = u16::from_str_radix(port_hex, 16)?;

        let addr = if addr_hex.len() == 8 {
            let a = u8::from_str_radix(&addr_hex[6..8], 16)?;
            let b = u8::from_str_radix(&addr_hex[4..6], 16)?;
            let c = u8::from_str_radix(&addr_hex[2..4], 16)?;
            let d = u8::from_str_radix(&addr_hex[0..2], 16)?;
            format!("{}.{}.{}.{}", a, b, c, d)
        } else {
            "::".to_string()
        };

        Ok((addr, port))
    }

    fn get_inode_to_pid_map(&self) -> HashMap<u64, u32> {
        let mut map = HashMap::new();

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<u32>() {
                        let fd_path = format!("/proc/{}/fd", pid);
                        if let Ok(fd_entries) = fs::read_dir(&fd_path) {
                            for fd_entry in fd_entries.flatten() {
                                if let Ok(link) = fs::read_link(fd_entry.path()) {
                                    if let Some(link_str) = link.to_str() {
                                        if link_str.starts_with("socket:[") {
                                            if let Some(inode_str) = link_str
                                                .strip_prefix("socket:[")
                                                .and_then(|s| s.strip_suffix(']'))
                                            {
                                                if let Ok(inode) = inode_str.parse::<u64>() {
                                                    map.insert(inode, pid);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        map
    }

    fn parse_proc_net(&mut self, path: &str, protocol: Protocol) -> Result<Vec<PortRecord>> {
        let content = fs::read_to_string(path)?;
        let inode_map = self.get_inode_to_pid_map();
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let mut records = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            if idx == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }

            let local = parts[1];
            let remote = parts[2];
            let state_hex = parts[3];
            let inode_str = parts[9];

            let (local_addr, local_port) = Self::parse_hex_addr(local).unwrap_or(("0.0.0.0".to_string(), 0));
            let (remote_addr, remote_port) = Self::parse_hex_addr(remote).unwrap_or(("0.0.0.0".to_string(), 0));

            let state = if protocol == Protocol::Tcp {
                Self::parse_tcp_state(state_hex)
            } else {
                ConnectionState::Listen
            };

            let inode = inode_str.parse::<u64>().ok();
            let pid = inode.and_then(|i| inode_map.get(&i).copied());

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

        Ok(records)
    }
}

impl PortBackend for LinuxPortBackend {
    fn scan_ports(&mut self) -> Result<Vec<PortRecord>> {
        let mut records = Vec::new();

        if let Ok(tcp_records) = self.parse_proc_net("/proc/net/tcp", Protocol::Tcp) {
            records.extend(tcp_records);
        }

        if let Ok(tcp6_records) = self.parse_proc_net("/proc/net/tcp6", Protocol::Tcp) {
            records.extend(tcp6_records);
        }

        if let Ok(udp_records) = self.parse_proc_net("/proc/net/udp", Protocol::Udp) {
            records.extend(udp_records);
        }

        if let Ok(udp6_records) = self.parse_proc_net("/proc/net/udp6", Protocol::Udp) {
            records.extend(udp6_records);
        }

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

pub struct LinuxProcessBackend {
    system: System,
}

impl LinuxProcessBackend {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

impl ProcessBackend for LinuxProcessBackend {
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
