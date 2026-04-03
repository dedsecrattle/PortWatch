use crate::models::{PortRecord, ProcessDetails};
use anyhow::Result;

pub trait PortBackend {
    fn scan_ports(&mut self) -> Result<Vec<PortRecord>>;
}

pub trait ProcessBackend {
    fn process_details(&mut self, pid: u32) -> Result<ProcessDetails>;
    fn stop_process(&mut self, pid: u32, graceful: bool) -> Result<()>;
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{LinuxPortBackend, LinuxProcessBackend};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{MacOsPortBackend, MacOsProcessBackend};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{WindowsPortBackend, WindowsProcessBackend};

pub struct Backend {
    port_backend: Box<dyn PortBackend>,
    process_backend: Box<dyn ProcessBackend>,
}

impl Backend {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            Self {
                port_backend: Box::new(LinuxPortBackend::new()),
                process_backend: Box::new(LinuxProcessBackend::new()),
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            Self {
                port_backend: Box::new(MacOsPortBackend::new()),
                process_backend: Box::new(MacOsProcessBackend::new()),
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            Self {
                port_backend: Box::new(WindowsPortBackend::new()),
                process_backend: Box::new(WindowsProcessBackend::new()),
            }
        }
    }
    
    pub fn scan_ports(&mut self) -> Result<Vec<PortRecord>> {
        self.port_backend.scan_ports()
    }
    
    pub fn process_details(&mut self, pid: u32) -> Result<ProcessDetails> {
        self.process_backend.process_details(pid)
    }
    
    pub fn stop_process(&mut self, pid: u32, graceful: bool) -> Result<()> {
        self.process_backend.stop_process(pid, graceful)
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
