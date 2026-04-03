use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

// Complete TCP/UDP connection state machine - not all states may be present at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ConnectionState {
    Listen,
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Closing,
    Unknown,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Listen => write!(f, "LISTEN"),
            ConnectionState::Established => write!(f, "ESTABLISHED"),
            ConnectionState::SynSent => write!(f, "SYN_SENT"),
            ConnectionState::SynRecv => write!(f, "SYN_RECV"),
            ConnectionState::FinWait1 => write!(f, "FIN_WAIT1"),
            ConnectionState::FinWait2 => write!(f, "FIN_WAIT2"),
            ConnectionState::TimeWait => write!(f, "TIME_WAIT"),
            ConnectionState::Close => write!(f, "CLOSE"),
            ConnectionState::CloseWait => write!(f, "CLOSE_WAIT"),
            ConnectionState::LastAck => write!(f, "LAST_ACK"),
            ConnectionState::Closing => write!(f, "CLOSING"),
            ConnectionState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortRecord {
    pub protocol: Protocol,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: Option<String>,
    pub remote_port: Option<u16>,
    pub state: ConnectionState,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub exe: Option<PathBuf>,
    pub cmdline: Vec<String>,
    pub user: Option<String>,
}

impl PortRecord {
    pub fn matches_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        
        let filter_lower = filter.to_lowercase();
        
        if let Ok(port) = filter.parse::<u16>() {
            if self.local_port == port {
                return true;
            }
        }
        
        if self.protocol.to_string().to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        if let Some(ref name) = self.process_name {
            if name.to_lowercase().contains(&filter_lower) {
                return true;
            }
        }
        
        if self.state.to_string().to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        false
    }
}

#[derive(Debug, Clone)]
pub struct ProcessDetails {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub exe: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub start_time: Option<u64>,
    pub cmdline: Vec<String>,
    pub env_preview: Vec<(String, String)>,
    pub user: Option<String>,
}

impl ProcessDetails {
    pub fn format_memory(&self) -> String {
        let bytes = self.memory_bytes as f64;
        if bytes < 1024.0 {
            format!("{} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
    
    pub fn format_uptime(&self) -> String {
        if let Some(start) = self.start_time {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let uptime = now.saturating_sub(start);
            
            let hours = uptime / 3600;
            let minutes = (uptime % 3600) / 60;
            let seconds = uptime % 60;
            
            if hours > 0 {
                format!("{}h {}m", hours, minutes)
            } else if minutes > 0 {
                format!("{}m {}s", minutes, seconds)
            } else {
                format!("{}s", seconds)
            }
        } else {
            "Unknown".to_string()
        }
    }
}
