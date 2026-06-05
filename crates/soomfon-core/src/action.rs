//! What a key does when it is pressed.
//!
//! A [`Button`](crate::Button) carries an [`Action`]. When the device session
//! sees a key go down it looks up that key's button and calls [`Action::run`].
//!
//! Actions are fire-and-forget: launching a command never blocks the input
//! loop, and the child is detached from our stdio so it can't write over the
//! app's console. A small reaper thread waits on the child so finished
//! processes don't pile up as zombies for the lifetime of the app.

use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::keyboard::{self, Keyboard, KeyboardError};

/// The behaviour bound to a key.
///
/// Serialized with an internal `"type"` tag so the on-disk shape is
/// self-describing and easy to extend with new variants (hotkeys, app launch,
/// OBS…) without breaking old files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// The key shows its label but does nothing when pressed.
    #[default]
    None,
    /// Launch an external program with the given arguments.
    RunCommand {
        /// The executable to run, resolved against `PATH`.
        program: String,
        /// Arguments passed verbatim to the program.
        #[serde(default)]
        args: Vec<String>,
    },
    /// Inject a keyboard shortcut into whatever window has focus.
    Hotkey {
        /// The key names making up the combo, e.g. `["ctrl", "shift", "m"]`.
        /// Matched case-insensitively against the virtual keyboard's table.
        keys: Vec<String>,
    },
}

/// A failure while carrying out an [`Action`].
#[derive(Debug, Error)]
pub enum ActionError {
    /// The program could not be launched (not found, not executable…).
    #[error("failed to launch `{program}`: {source}")]
    Spawn {
        /// The program we tried to run.
        program: String,
        /// The underlying OS error.
        source: std::io::Error,
    },
    /// A hotkey was triggered but no virtual keyboard is available — usually the
    /// session couldn't open `/dev/uinput` at startup (missing permission).
    #[error("no virtual keyboard available for the hotkey")]
    NoKeyboard,
    /// The virtual keyboard rejected or failed to inject the hotkey.
    #[error(transparent)]
    Keyboard(#[from] KeyboardError),
}

impl Action {
    /// Carry out the action.
    ///
    /// Returns as soon as the work is launched; it does not wait for a spawned
    /// command to finish. [`Action::None`] is always a no-op. A [`Action::Hotkey`]
    /// needs the session's shared `keyboard`; passing `None` makes it fail with
    /// [`ActionError::NoKeyboard`] rather than silently doing nothing.
    pub fn run(&self, keyboard: Option<&mut Keyboard>) -> Result<(), ActionError> {
        match self {
            Action::None => Ok(()),
            Action::RunCommand { program, args } => {
                let child = Command::new(program)
                    .args(args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(|source| ActionError::Spawn {
                        program: program.clone(),
                        source,
                    })?;
                reap(child);
                Ok(())
            }
            Action::Hotkey { keys } => {
                let keyboard = keyboard.ok_or(ActionError::NoKeyboard)?;
                let combo = keyboard::parse_combo(keys)?;
                keyboard.tap(&combo)?;
                Ok(())
            }
        }
    }
}

/// Wait on a launched child on a throwaway thread so it is reaped once it exits.
fn reap(mut child: std::process::Child) {
    std::thread::spawn(move || {
        let _ = child.wait();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_the_default_and_a_noop() {
        assert_eq!(Action::default(), Action::None);
        assert!(Action::None.run(None).is_ok());
    }

    #[test]
    fn run_command_launches_an_existing_program() {
        // `true` exists on every supported (Linux) target and exits cleanly.
        let action = Action::RunCommand {
            program: "true".into(),
            args: vec![],
        };
        assert!(action.run(None).is_ok());
    }

    #[test]
    fn run_command_reports_a_missing_program() {
        let action = Action::RunCommand {
            program: "soomfon-no-such-binary-zzz".into(),
            args: vec![],
        };
        assert!(matches!(action.run(None), Err(ActionError::Spawn { .. })));
    }

    #[test]
    fn hotkey_without_a_keyboard_reports_it() {
        // Opening a real uinput device needs privileges the test runner lacks,
        // so this only checks the missing-keyboard path.
        let action = Action::Hotkey {
            keys: vec!["ctrl".into(), "c".into()],
        };
        assert!(matches!(action.run(None), Err(ActionError::NoKeyboard)));
    }

    #[test]
    fn hotkey_round_trips_through_json() {
        let action = Action::Hotkey {
            keys: vec!["ctrl".into(), "shift".into(), "m".into()],
        };
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, r#"{"type":"hotkey","keys":["ctrl","shift","m"]}"#);
        assert_eq!(serde_json::from_str::<Action>(&json).unwrap(), action);
    }

    #[test]
    fn action_round_trips_through_json() {
        let action = Action::RunCommand {
            program: "obs".into(),
            args: vec!["--minimize-to-tray".into()],
        };
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(
            json,
            r#"{"type":"run_command","program":"obs","args":["--minimize-to-tray"]}"#
        );
        assert_eq!(serde_json::from_str::<Action>(&json).unwrap(), action);
    }

    #[test]
    fn none_serializes_to_a_tagged_object() {
        assert_eq!(
            serde_json::to_string(&Action::None).unwrap(),
            r#"{"type":"none"}"#
        );
    }

    #[test]
    fn run_command_args_default_to_empty() {
        let action: Action =
            serde_json::from_str(r#"{"type":"run_command","program":"firefox"}"#).unwrap();
        assert_eq!(
            action,
            Action::RunCommand {
                program: "firefox".into(),
                args: vec![]
            }
        );
    }
}
