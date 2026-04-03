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

        // Windows: notify-rust uses the WinRT backend; `.urgency()` exists only on Freedesktop (Linux, etc.).
        #[cfg(target_os = "windows")]
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

        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
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

            macos_send(summary, &alert.message)?;
        }

        Ok(())
    }
}

/// Prefer `osascript display notification` for CLI binaries: it tends to behave more reliably when
/// the app is not a signed `.app` bundle. Fall back to `notify-rust` if `osascript` is missing or
/// errors.
#[cfg(target_os = "macos")]
fn macos_send(summary: &str, message: &str) -> anyhow::Result<()> {
    let body = truncate_for_notification(message);
    match macos_osascript_notify(summary, &body) {
        Ok(()) => Ok(()),
        Err(e_os) => Notification::new()
            .summary(summary)
            .body(&body)
            .timeout(Timeout::Milliseconds(5000))
            .show()
            .map(|_| ())
            .map_err(|e_nr| {
                anyhow::anyhow!("osascript: {}; notify-rust: {}", e_os, e_nr)
            }),
    }
}

#[cfg(target_os = "macos")]
fn truncate_for_notification(s: &str) -> String {
    const MAX_CHARS: usize = 800;
    if s.chars().count() <= MAX_CHARS {
        return s.to_string();
    }
    s.chars().take(MAX_CHARS.saturating_sub(1)).collect::<String>() + "…"
}

#[cfg(target_os = "macos")]
fn macos_osascript_notify(title: &str, body: &str) -> anyhow::Result<()> {
    use std::process::Command;

    let script = format!(
        "display notification {} with title {}",
        applescript_string_literal(body),
        applescript_string_literal(title)
    );
    let output = Command::new("osascript").args(["-e", &script]).output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!(
        "{}",
        stderr.trim().trim_start_matches("osascript: ").trim()
    );
}

/// Escape text for a double-quoted AppleScript string literal.
#[cfg(target_os = "macos")]
fn applescript_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new(true)
    }
}
