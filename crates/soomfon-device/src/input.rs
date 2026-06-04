//! Decoding raw device reports into high-level input events.
//!
//! The raw byte values come straight from the OEM firmware and are shared
//! across the whole AKP03 / N3 / Soomfon family, so this logic is model
//! independent.

use mirajazz::{error::MirajazzError, state::DeviceStateUpdate, types::DeviceInput};

const KEY_COUNT: usize = 9;
const ENCODER_COUNT: usize = 3;

/// A high-level input event from a deck, decoupled from the USB transport so the
/// rest of the application never depends on `mirajazz` types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// A key was pressed down. `key` is 0-based, row-major across the 3x3 grid.
    KeyDown(u8),
    /// A key was released.
    KeyUp(u8),
    /// An encoder was pressed in.
    EncoderDown(u8),
    /// An encoder was released.
    EncoderUp(u8),
    /// An encoder was rotated; the value is the signed step count (positive is
    /// clockwise).
    EncoderTwist(u8, i8),
}

impl InputEvent {
    pub(crate) fn from_update(update: DeviceStateUpdate) -> Self {
        match update {
            DeviceStateUpdate::ButtonDown(key) => Self::KeyDown(key),
            DeviceStateUpdate::ButtonUp(key) => Self::KeyUp(key),
            DeviceStateUpdate::EncoderDown(encoder) => Self::EncoderDown(encoder),
            DeviceStateUpdate::EncoderUp(encoder) => Self::EncoderUp(encoder),
            DeviceStateUpdate::EncoderTwist(encoder, value) => Self::EncoderTwist(encoder, value),
        }
    }
}

/// Decode a single raw `(input, state)` report.
///
/// `mirajazz` calls this for every report it reads and diffs the result against
/// the previous state to emit press / release / twist updates.
pub(crate) fn process_input(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    match input {
        0 | 1..=6 | 0x25 | 0x30 | 0x31 => {
            Ok(DeviceInput::ButtonStateChange(read_buttons(input, state)))
        }
        0x90 | 0x91 | 0x50 | 0x51 | 0x60 | 0x61 => Ok(DeviceInput::EncoderTwist(read_twist(input))),
        0x33..=0x35 => Ok(DeviceInput::EncoderStateChange(read_encoder_press(
            input, state,
        ))),
        _ => Err(MirajazzError::BadData),
    }
}

/// Build the full 9-key pressed/released snapshot for a button report.
///
/// `input == 0` is the firmware's "nothing pressed" snapshot. The six LCD keys
/// report as `1..=6`; the three plain keys report as `0x25`, `0x30`, `0x31`.
fn read_buttons(input: u8, state: u8) -> Vec<bool> {
    let mut keys = vec![false; KEY_COUNT];

    let pressed = match input {
        1..=6 => Some(input as usize),
        0x25 => Some(7),
        0x30 => Some(8),
        0x31 => Some(9),
        _ => None,
    };

    if let Some(one_based) = pressed {
        keys[one_based - 1] = state != 0;
    }

    keys
}

/// Decode a single-step encoder rotation into a per-encoder delta vector.
fn read_twist(input: u8) -> Vec<i8> {
    let mut values = vec![0i8; ENCODER_COUNT];

    let (encoder, delta) = match input {
        0x90 => (0, -1),
        0x91 => (0, 1),
        0x50 => (1, -1),
        0x51 => (1, 1),
        0x60 => (2, -1),
        0x61 => (2, 1),
        _ => return values,
    };

    values[encoder] = delta;
    values
}

/// Decode an encoder push/release into a per-encoder pressed-state vector.
fn read_encoder_press(input: u8, state: u8) -> Vec<bool> {
    let mut states = vec![false; ENCODER_COUNT];

    let encoder = match input {
        0x33 => 0,
        0x35 => 1,
        0x34 => 2,
        _ => return states,
    };

    states[encoder] = state != 0;
    states
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buttons(input: u8, state: u8) -> Vec<bool> {
        match process_input(input, state).unwrap() {
            DeviceInput::ButtonStateChange(b) => b,
            other => panic!("expected ButtonStateChange, got {other:?}"),
        }
    }

    fn twist(input: u8) -> Vec<i8> {
        match process_input(input, 0).unwrap() {
            DeviceInput::EncoderTwist(v) => v,
            other => panic!("expected EncoderTwist, got {other:?}"),
        }
    }

    #[test]
    fn first_lcd_key_press_sets_key_zero() {
        let b = buttons(1, 1);
        assert!(b[0]);
        assert_eq!(b.iter().filter(|&&p| p).count(), 1);
    }

    #[test]
    fn plain_keys_map_to_last_three_slots() {
        assert!(buttons(0x25, 1)[6]);
        assert!(buttons(0x30, 1)[7]);
        assert!(buttons(0x31, 1)[8]);
    }

    #[test]
    fn zero_report_is_all_released() {
        assert!(buttons(0, 1).iter().all(|&p| !p));
    }

    #[test]
    fn encoder_rotation_is_signed_per_encoder() {
        assert_eq!(twist(0x90), vec![-1, 0, 0]);
        assert_eq!(twist(0x91), vec![1, 0, 0]);
        assert_eq!(twist(0x61), vec![0, 0, 1]);
    }

    #[test]
    fn encoder_press_maps_0x35_to_middle() {
        match process_input(0x35, 1).unwrap() {
            DeviceInput::EncoderStateChange(s) => assert_eq!(s, vec![false, true, false]),
            other => panic!("expected EncoderStateChange, got {other:?}"),
        }
    }

    #[test]
    fn unknown_report_is_bad_data() {
        assert!(matches!(
            process_input(0xAA, 0),
            Err(MirajazzError::BadData)
        ));
    }
}
