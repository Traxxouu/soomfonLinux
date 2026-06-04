//! Core application logic for soomfonLinux.
//!
//! This crate sits between the hardware layer ([`soomfon_device`]) and the
//! desktop frontend. It owns the application state that the UI renders and the
//! actions a key press can trigger. Alongside the live [`status`] snapshot it
//! owns the persisted user [`config`] — profiles, pages and per-key buttons.
//! Action dispatch lands in a later branch (`feat/action-*`).

pub mod config;

pub use config::{Button, Config, ConfigError, ConfigStore, Page, Profile};

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
pub async fn status() -> Status {
    let devices = soomfon_device::list_devices().await.unwrap_or_default();
    Status::from_devices(devices)
}

impl Status {
    fn from_devices(devices: Vec<soomfon_device::DeviceInfo>) -> Status {
        Status {
            version: env!("CARGO_PKG_VERSION").to_string(),
            devices: devices
                .into_iter()
                .map(|d| DeviceSummary {
                    model: d.model,
                    keys: d.keys,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_version_and_summarizes_devices() {
        let s = Status::from_devices(vec![soomfon_device::DeviceInfo {
            model: "Soomfon Stream Controller SE".into(),
            vid: 0x1500,
            pid: 0x3001,
            keys: 9,
            encoders: 3,
            has_key_screens: true,
            serial: None,
        }]);

        assert!(!s.version.is_empty());
        assert_eq!(s.devices.len(), 1);
        assert_eq!(s.devices[0].keys, 9);
    }
}
