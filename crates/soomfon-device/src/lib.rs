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
pub mod render;

use std::sync::Arc;

use mirajazz::{device::Device, state::DeviceStateReader};
use thiserror::Error;

pub use image::{DynamicImage, RgbImage};
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
    /// A drawing call targeted a key that has no LCD screen.
    #[error("key {key} has no screen (only the first {screens} keys do)")]
    NoScreen {
        /// The out-of-range key index.
        key: u8,
        /// How many keys actually have a screen.
        screens: u8,
    },
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
/// Connecting takes the device out of its standalone mode and into host mode:
/// it sets a default brightness and clears the key screens. This handshake is
/// what makes the deck stream key/encoder events to us instead of handling
/// presses on its own.
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

            let deck = Deck { device, kind };
            deck.initialize().await?;

            return Ok(Some(deck));
        }

        Ok(None)
    }

    /// Run the host handshake: without it the deck stays in standalone mode and
    /// never reports input. Blanks the key screens as a side effect.
    async fn initialize(&self) -> Result<(), DeviceError> {
        self.device.set_brightness(50).await.map_err(transport)?;
        self.device
            .clear_all_button_images()
            .await
            .map_err(transport)?;
        self.device.flush().await.map_err(transport)?;
        Ok(())
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

    /// Number of keys that can show an image (the rest are plain buttons).
    pub fn lcd_key_count(&self) -> u8 {
        self.kind.lcd_key_count()
    }

    /// Set the panel brightness, `0..=100` percent.
    pub async fn set_brightness(&self, percent: u8) -> Result<(), DeviceError> {
        self.device.set_brightness(percent).await.map_err(transport)
    }

    /// Queue an image for a key. The bitmap is resized and re-encoded to the
    /// device's native format automatically.
    ///
    /// Drawing is buffered: nothing changes on the panel until [`Deck::flush`]
    /// is called, so several keys can be updated in one batch.
    pub async fn set_key_image(&self, key: u8, image: DynamicImage) -> Result<(), DeviceError> {
        self.check_screen(key)?;
        self.device
            .set_button_image(key, self.kind.image_format(), image)
            .await
            .map_err(transport)
    }

    /// Queue a solid-colour fill for a key. Buffered; see [`Deck::set_key_image`].
    pub async fn set_key_color(&self, key: u8, rgb: [u8; 3]) -> Result<(), DeviceError> {
        let image = render::solid(self.kind.key_image_size(), rgb);
        self.set_key_image(key, DynamicImage::ImageRgb8(image))
            .await
    }

    /// Queue a short text label for a key, `fg` on `bg`. Newlines split lines.
    /// Buffered; see [`Deck::set_key_image`].
    pub async fn set_key_text(
        &self,
        key: u8,
        text: &str,
        fg: [u8; 3],
        bg: [u8; 3],
    ) -> Result<(), DeviceError> {
        let image = render::text(self.kind.key_image_size(), text, fg, bg);
        self.set_key_image(key, DynamicImage::ImageRgb8(image))
            .await
    }

    /// Queue a key to be blanked. Buffered; see [`Deck::set_key_image`].
    pub async fn clear_key(&self, key: u8) -> Result<(), DeviceError> {
        self.check_screen(key)?;
        self.device.clear_button_image(key).await.map_err(transport)
    }

    /// Queue every key to be blanked. Buffered; see [`Deck::set_key_image`].
    pub async fn clear_all_keys(&self) -> Result<(), DeviceError> {
        self.device
            .clear_all_button_images()
            .await
            .map_err(transport)
    }

    /// Push all buffered drawing to the panel.
    pub async fn flush(&self) -> Result<(), DeviceError> {
        self.device.flush().await.map_err(transport)
    }

    fn check_screen(&self, key: u8) -> Result<(), DeviceError> {
        let screens = self.kind.lcd_key_count();
        if key < screens {
            Ok(())
        } else {
            Err(DeviceError::NoScreen { key, screens })
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
    /// a report that produced no state change, or one we don't recognize.
    ///
    /// The deck interleaves housekeeping reports (acks, heartbeats) with input.
    /// `mirajazz` flags the ones it can't decode as [`MirajazzError::BadData`];
    /// those are skipped rather than treated as fatal, so a single stray report
    /// never tears down the read loop.
    pub async fn next_events(&self) -> Result<Vec<InputEvent>, DeviceError> {
        match self.inner.read(None).await {
            Ok(updates) => Ok(updates.into_iter().map(InputEvent::from_update).collect()),
            Err(mirajazz::error::MirajazzError::BadData) => Ok(Vec::new()),
            Err(err) => Err(transport(err)),
        }
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
