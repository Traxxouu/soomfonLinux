//! Hardware layer for Soomfon stream decks.
//!
//! Soomfon devices are OEM rebrands of Mirabox / Ajazz hardware, so the USB
//! protocol is shared with those families. The actual transport will be backed
//! by the [`mirajazz`](https://github.com/4ndv/mirajazz) crate; that wiring
//! lands in the `feat/device-detection` branch once we have the exact VID/PID.
//!
//! This crate intentionally exposes a transport-agnostic surface
//! ([`DeviceInfo`], [`list_devices`]) so the rest of the application never
//! depends on USB details directly.

use thiserror::Error;

/// Errors raised while discovering or talking to a deck.
#[derive(Debug, Error)]
pub enum DeviceError {
    /// No supported Soomfon device is currently connected.
    #[error("no supported Soomfon device found")]
    NotFound,
    /// The underlying USB/HID transport failed.
    #[error("device transport error: {0}")]
    Transport(String),
}

/// Static description of a connected, supported deck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Human-readable model name (e.g. "Soomfon Stream Controller").
    pub model: String,
    /// USB vendor id.
    pub vid: u16,
    /// USB product id.
    pub pid: u16,
    /// Number of physical keys.
    pub keys: u8,
    /// Number of rotary encoders / knobs.
    pub encoders: u8,
    /// Whether the keys are individually addressable LCD screens.
    pub has_key_screens: bool,
}

/// Enumerate every connected, supported deck.
///
/// Returns an empty list until the `mirajazz` transport is wired in
/// (`feat/device-detection`). It never errors today, but keeps a [`Result`]
/// signature so callers are already written against the final contract.
pub fn list_devices() -> Result<Vec<DeviceInfo>, DeviceError> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_devices_is_empty_until_transport_is_wired() {
        assert!(list_devices().unwrap().is_empty());
    }
}
