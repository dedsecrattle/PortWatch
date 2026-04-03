use super::{Alert, AlertSeverity};
use notify_rust::{Notification, Timeout};

pub struct Notifier {
    enabled: bool,
}

impl Notifier {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn send(&self, alert: &Alert) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(not(target_os = "macos"))]
        {
            let (summary, urgency) = match alert.severity {
                AlertSeverity::Info => ("PortWatch - Info", notify_rust::Urgency::Low),
                AlertSeverity::Warning => ("PortWatch - Warning", notify_rust::Urgency::Normal),
                AlertSeverity::Critical => ("PortWatch - Critical", notify_rust::Urgency::Critical),
            };
            
            Notification::new()
                .summary(summary)
                .body(&alert.message)
                .urgency(urgency)
                .timeout(Timeout::Milliseconds(5000))
                .show()?;
        }

        #[cfg(target_os = "macos")]
        {
            let summary = match alert.severity {
                AlertSeverity::Info => "PortWatch - Info",
                AlertSeverity::Warning => "PortWatch - Warning",
                AlertSeverity::Critical => "PortWatch - Critical",
            };
            
            Notification::new()
                .summary(summary)
                .body(&alert.message)
                .timeout(Timeout::Milliseconds(5000))
                .show()?;
        }

        Ok(())
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new(true)
    }
}
