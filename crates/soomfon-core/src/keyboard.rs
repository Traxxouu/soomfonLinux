//! A virtual keyboard for injecting hotkeys into the focused window.
//!
//! Wayland (and a locked-down X11) won't let an ordinary process synthesize key
//! presses through the usual desktop APIs, so we go one level lower: a uinput
//! virtual device registered with the kernel. To the rest of the system it looks
//! like a real keyboard, which makes the injected keys land in whatever window
//! currently has focus, on both X11 and Wayland.
//!
//! Opening the device needs write access to `/dev/uinput` (see the udev rule and
//! the README setup). When that access is missing, [`Keyboard::open`] fails and
//! the caller is expected to carry on without hotkey support rather than refuse
//! to start.

use std::io;

use evdev::{uinput::VirtualDevice, AttributeSet, InputEvent, KeyCode, KeyEvent};
use thiserror::Error;

/// Hotkey names accepted in a combo, each mapped to its Linux key code.
///
/// Names are matched case-insensitively. Modifiers and ordinary keys share one
/// table, so `"ctrl+shift+m"` is just three lookups. Codes are referenced by
/// their [`KeyCode`] constant rather than a number because the letter row is
/// laid out in QWERTY order, not alphabetically, so a code can't be derived from
/// its character.
const KEYMAP: &[(&str, KeyCode)] = &[
    // Modifiers — the left-hand variant is used for each.
    ("ctrl", KeyCode::KEY_LEFTCTRL),
    ("control", KeyCode::KEY_LEFTCTRL),
    ("shift", KeyCode::KEY_LEFTSHIFT),
    ("alt", KeyCode::KEY_LEFTALT),
    ("super", KeyCode::KEY_LEFTMETA),
    ("meta", KeyCode::KEY_LEFTMETA),
    ("win", KeyCode::KEY_LEFTMETA),
    // Letters.
    ("a", KeyCode::KEY_A),
    ("b", KeyCode::KEY_B),
    ("c", KeyCode::KEY_C),
    ("d", KeyCode::KEY_D),
    ("e", KeyCode::KEY_E),
    ("f", KeyCode::KEY_F),
    ("g", KeyCode::KEY_G),
    ("h", KeyCode::KEY_H),
    ("i", KeyCode::KEY_I),
    ("j", KeyCode::KEY_J),
    ("k", KeyCode::KEY_K),
    ("l", KeyCode::KEY_L),
    ("m", KeyCode::KEY_M),
    ("n", KeyCode::KEY_N),
    ("o", KeyCode::KEY_O),
    ("p", KeyCode::KEY_P),
    ("q", KeyCode::KEY_Q),
    ("r", KeyCode::KEY_R),
    ("s", KeyCode::KEY_S),
    ("t", KeyCode::KEY_T),
    ("u", KeyCode::KEY_U),
    ("v", KeyCode::KEY_V),
    ("w", KeyCode::KEY_W),
    ("x", KeyCode::KEY_X),
    ("y", KeyCode::KEY_Y),
    ("z", KeyCode::KEY_Z),
    // Digit row.
    ("0", KeyCode::KEY_0),
    ("1", KeyCode::KEY_1),
    ("2", KeyCode::KEY_2),
    ("3", KeyCode::KEY_3),
    ("4", KeyCode::KEY_4),
    ("5", KeyCode::KEY_5),
    ("6", KeyCode::KEY_6),
    ("7", KeyCode::KEY_7),
    ("8", KeyCode::KEY_8),
    ("9", KeyCode::KEY_9),
    // Function keys.
    ("f1", KeyCode::KEY_F1),
    ("f2", KeyCode::KEY_F2),
    ("f3", KeyCode::KEY_F3),
    ("f4", KeyCode::KEY_F4),
    ("f5", KeyCode::KEY_F5),
    ("f6", KeyCode::KEY_F6),
    ("f7", KeyCode::KEY_F7),
    ("f8", KeyCode::KEY_F8),
    ("f9", KeyCode::KEY_F9),
    ("f10", KeyCode::KEY_F10),
    ("f11", KeyCode::KEY_F11),
    ("f12", KeyCode::KEY_F12),
    // Whitespace and editing.
    ("enter", KeyCode::KEY_ENTER),
    ("return", KeyCode::KEY_ENTER),
    ("space", KeyCode::KEY_SPACE),
    ("tab", KeyCode::KEY_TAB),
    ("esc", KeyCode::KEY_ESC),
    ("escape", KeyCode::KEY_ESC),
    ("backspace", KeyCode::KEY_BACKSPACE),
    ("delete", KeyCode::KEY_DELETE),
    ("del", KeyCode::KEY_DELETE),
    ("insert", KeyCode::KEY_INSERT),
    // Navigation.
    ("up", KeyCode::KEY_UP),
    ("down", KeyCode::KEY_DOWN),
    ("left", KeyCode::KEY_LEFT),
    ("right", KeyCode::KEY_RIGHT),
    ("home", KeyCode::KEY_HOME),
    ("end", KeyCode::KEY_END),
    ("pageup", KeyCode::KEY_PAGEUP),
    ("pagedown", KeyCode::KEY_PAGEDOWN),
];

/// A failure while setting up or using the virtual keyboard.
#[derive(Debug, Error)]
pub enum KeyboardError {
    /// The uinput virtual device could not be created — usually missing write
    /// permission on `/dev/uinput`.
    #[error("could not open the virtual keyboard (is /dev/uinput writable?): {0}")]
    Open(#[source] io::Error),
    /// Writing the key events to the virtual device failed.
    #[error("could not inject keys: {0}")]
    Emit(#[source] io::Error),
    /// A combo referenced a key name that isn't in [`KEYMAP`].
    #[error("unknown key in hotkey: `{0}`")]
    UnknownKey(String),
}

/// A uinput-backed virtual keyboard.
///
/// Created once and reused for the lifetime of the session: registering a new
/// device per key press would be slow and races the kernel's settling of the
/// node, which can swallow the first event.
pub struct Keyboard {
    device: VirtualDevice,
}

impl Keyboard {
    /// Register a virtual keyboard with the kernel.
    ///
    /// The device advertises every key in [`KEYMAP`] up front so any combo can be
    /// emitted later without re-opening it.
    pub fn open() -> Result<Self, KeyboardError> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for (_, code) in KEYMAP {
            keys.insert(*code);
        }
        let device = VirtualDevice::builder()
            .map_err(KeyboardError::Open)?
            .name("soomfonLinux virtual keyboard")
            .with_keys(&keys)
            .map_err(KeyboardError::Open)?
            .build()
            .map_err(KeyboardError::Open)?;
        Ok(Self { device })
    }

    /// Tap a combo: hold every key down in order, then release in reverse.
    ///
    /// The presses go out as one batch and the releases as another, so the kernel
    /// sees every modifier still held when the final key lands (`emit` terminates
    /// each batch with its own `SYN_REPORT`).
    pub fn tap(&mut self, keys: &[KeyCode]) -> Result<(), KeyboardError> {
        let down: Vec<InputEvent> = keys.iter().map(|k| key_event(*k, 1)).collect();
        let up: Vec<InputEvent> = keys.iter().rev().map(|k| key_event(*k, 0)).collect();
        self.device.emit(&down).map_err(KeyboardError::Emit)?;
        self.device.emit(&up).map_err(KeyboardError::Emit)?;
        Ok(())
    }
}

/// Build a single key press (`value == 1`) or release (`value == 0`) event.
fn key_event(code: KeyCode, value: i32) -> InputEvent {
    KeyEvent::new(code, value).into()
}

/// Resolve a combo of key names to their key codes, in order.
///
/// Each name is trimmed and matched case-insensitively. Returns
/// [`KeyboardError::UnknownKey`] for the first name that isn't recognised.
pub fn parse_combo(names: &[String]) -> Result<Vec<KeyCode>, KeyboardError> {
    names.iter().map(|name| key_for(name)).collect()
}

/// Look up a single key name (trimmed, case-insensitive) in [`KEYMAP`].
fn key_for(name: &str) -> Result<KeyCode, KeyboardError> {
    let needle = name.trim().to_ascii_lowercase();
    KEYMAP
        .iter()
        .find(|(candidate, _)| *candidate == needle)
        .map(|(_, code)| *code)
        .ok_or_else(|| KeyboardError::UnknownKey(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests only exercise name parsing; opening a real uinput device needs
    // privileges the CI runner doesn't have, so `Keyboard::open`/`tap` are left
    // to the manual hardware test.

    #[test]
    fn parse_combo_resolves_a_modifier_chord() {
        let combo = vec!["ctrl".into(), "shift".into(), "m".into()];
        assert_eq!(
            parse_combo(&combo).unwrap(),
            vec![
                KeyCode::KEY_LEFTCTRL,
                KeyCode::KEY_LEFTSHIFT,
                KeyCode::KEY_M
            ]
        );
    }

    #[test]
    fn key_names_are_case_insensitive_and_trimmed() {
        assert_eq!(key_for(" Ctrl ").unwrap(), KeyCode::KEY_LEFTCTRL);
        assert_eq!(key_for("ENTER").unwrap(), KeyCode::KEY_ENTER);
    }

    #[test]
    fn aliases_map_to_the_same_code() {
        assert_eq!(key_for("super").unwrap(), key_for("win").unwrap());
        assert_eq!(key_for("esc").unwrap(), key_for("escape").unwrap());
        assert_eq!(key_for("del").unwrap(), key_for("delete").unwrap());
    }

    #[test]
    fn an_unknown_name_is_reported() {
        assert!(matches!(
            key_for("nope"),
            Err(KeyboardError::UnknownKey(name)) if name == "nope"
        ));
    }
}
