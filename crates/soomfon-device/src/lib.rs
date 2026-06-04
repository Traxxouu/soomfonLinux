//! Hardware layer for Soomfon stream decks.
//!
//! Soomfon devices are OEM rebrands of Mirabox / Ajazz hardware, so the USB
//! protocol is shared with those families. The transport is backed by the
//! [`mirajazz`](https://github.com/4ndv/mirajazz) crate.
//!
//! This crate exposes a transport-agnostic surface ([`DeviceInfo`],
//! [`list_devices`], [`Deck`], [`InputEvent`]) so the rest of the application
//! never depends on USB details directly.

mod input;
mod kind;

use std::sync::Arc;

use mirajazz::{device::Device, state::DeviceStateReader};
use thiserror::Error;

pub use input::InputEvent;
pub use kind::Kind;

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

fn transport(err: mirajazz::error::MirajazzError) -> DeviceError {
    DeviceError::Transport(err.to_string())
}

/// Static description of a connected, supported deck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Human-readable model name (e.g. "Soomfon Stream Controller SE").
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
    /// USB serial number, when the device reports one.
    pub serial: Option<String>,
}

impl DeviceInfo {
    fn from_kind(kind: Kind, vid: u16, pid: u16, serial: Option<String>) -> Self {
        Self {
            model: kind.model().to_string(),
            vid,
            pid,
            keys: kind.key_count(),
            encoders: kind.encoder_count(),
            has_key_screens: kind.has_key_screens(),
            serial,
        }
    }
}

/// Enumerate every connected, supported deck.
///
/// This only reads HID metadata; it does not open or alter the devices.
pub async fn list_devices() -> Result<Vec<DeviceInfo>, DeviceError> {
    let queries = kind::queries();
    let raw = mirajazz::device::list_devices(&queries)
        .await
        .map_err(transport)?;

    let mut devices = Vec::new();
    for info in raw {
        if let Some(kind) = Kind::from_ids(info.vendor_id, info.product_id) {
            devices.push(DeviceInfo::from_kind(
                kind,
                info.vendor_id,
                info.product_id,
                info.serial_number.clone(),
            ));
        }
    }

    Ok(devices)
}

/// A connected deck, ready to read input from.
///
/// Connecting does not write to the device (it leaves the screens and
/// brightness untouched); rendering lands in a later branch.
pub struct Deck {
    device: Device,
    kind: Kind,
}

impl Deck {
    /// Connect to the first supported deck found, or `Ok(None)` if none is
    /// plugged in.
    pub async fn connect_first() -> Result<Option<Deck>, DeviceError> {
        let queries = kind::queries();
        let raw = mirajazz::device::list_devices(&queries)
            .await
            .map_err(transport)?;

        for info in raw {
            let Some(kind) = Kind::from_ids(info.vendor_id, info.product_id) else {
                continue;
            };

            let device = Device::connect(
                &info,
                kind.is_v2(),
                kind.supports_both_states(),
                kind.key_count() as usize,
                kind.encoder_count() as usize,
            )
            .await
            .map_err(transport)?;

            return Ok(Some(Deck { device, kind }));
        }

        Ok(None)
    }

    /// The resolved model of this deck.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Human-readable model name.
    pub fn model(&self) -> &'static str {
        self.kind.model()
    }

    /// USB serial number reported by the device.
    pub fn serial(&self) -> &str {
        self.device.serial_number()
    }

    /// Open a reader that yields decoded [`InputEvent`]s.
    pub fn reader(&self) -> EventReader {
        EventReader {
            inner: self.device.get_reader(input::process_input),
        }
    }
}

/// Reads and decodes input events from a connected [`Deck`].
pub struct EventReader {
    inner: Arc<DeviceStateReader>,
}

impl EventReader {
    /// Wait for the next batch of input events.
    ///
    /// A single physical action (a key press, an encoder click) maps to one or
    /// more [`InputEvent`]s. The returned vector may be empty if the device sent
    /// a report that produced no state change.
    pub async fn next_events(&self) -> Result<Vec<InputEvent>, DeviceError> {
        let updates = self.inner.read(None).await.map_err(transport)?;
        Ok(updates.into_iter().map(InputEvent::from_update).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_info_is_built_from_the_model_registry() {
        let info = DeviceInfo::from_kind(Kind::SoomfonSe, 0x1500, 0x3001, Some("abc".into()));
        assert_eq!(info.model, "Soomfon Stream Controller SE");
        assert_eq!(info.keys, 9);
        assert_eq!(info.encoders, 3);
        assert!(info.has_key_screens);
        assert_eq!(info.serial.as_deref(), Some("abc"));
    }
}
