//! Core application logic for soomfonLinux.
//!
//! This crate sits between the hardware layer ([`soomfon_device`]) and the
//! desktop frontend. It owns the application state that the UI renders and the
//! actions a key press can trigger. For now it only exposes a [`status`]
//! snapshot used to prove the layering end to end; profiles, pages and actions
//! land in later branches (`feat/profiles-config`, `feat/action-*`).

use serde::{Deserialize, Serialize};

/// A serializable snapshot of the application state, sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Crate version, surfaced in the UI.
    pub version: String,
    /// Currently connected, supported decks.
    pub devices: Vec<DeviceSummary>,
}

/// A frontend-friendly summary of a connected deck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSummary {
    /// Human-readable model name.
    pub model: String,
    /// Number of physical keys.
    pub keys: u8,
}

/// Build a [`Status`] snapshot from the current hardware state.
pub fn status() -> Status {
    let devices = soomfon_device::list_devices()
        .unwrap_or_default()
        .into_iter()
        .map(|d| DeviceSummary {
            model: d.model,
            keys: d.keys,
        })
        .collect();

    Status {
        version: env!("CARGO_PKG_VERSION").to_string(),
        devices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_version_and_no_devices_yet() {
        let s = status();
        assert!(!s.version.is_empty());
        assert!(s.devices.is_empty());
    }
}
