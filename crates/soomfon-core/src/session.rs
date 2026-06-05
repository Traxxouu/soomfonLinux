//! The device session: connect to a deck, mirror the active page onto it, and
//! run each key's action when it is pressed.
//!
//! This is the runtime that ties the persisted [`config`](crate::config) to the
//! hardware. The desktop backend spawns [`run_device_session`] once at startup
//! and the session looks after connecting, drawing and dispatching on its own.

use std::sync::Arc;
use std::time::Duration;

use soomfon_device::{Deck, DeviceError, InputEvent};
use tokio::sync::Notify;

use crate::config::{self, Button};
use crate::keyboard::Keyboard;

/// How long to wait before retrying after a disconnect or a connection error.
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

/// A nudge asking the session to repaint the deck from the latest config.
///
/// The session draws the active page when it connects and then only reacts to
/// key presses, so edits saved from the UI wouldn't show on the panel until a
/// reconnect. Triggering this signal after a save makes the change appear at
/// once. Cloning shares the same underlying notification.
#[derive(Clone, Default)]
pub struct RedrawSignal(Arc<Notify>);

impl RedrawSignal {
    /// Create a fresh signal with no pending redraw.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask the running session to repaint the deck.
    ///
    /// If the session is mid-repaint or busy, the request is coalesced: a single
    /// repaint with the newest config still happens, so rapid saves don't queue
    /// up redundant redraws.
    pub fn trigger(&self) {
        self.0.notify_one();
    }

    /// Wait for the next redraw request.
    async fn wait(&self) {
        self.0.notified().await;
    }
}

/// Run the device session forever: (re)connect to a deck and dispatch input.
///
/// This never returns. When no deck is present, or the connection drops, it
/// waits [`RECONNECT_DELAY`] and tries again, so plugging the device in (or back
/// in) is enough to start driving it.
pub async fn run_device_session(redraw: RedrawSignal) {
    // Open the virtual keyboard once, up front, and reuse it for the whole
    // session: registering one per key press would be slow and races the kernel
    // settling the node. If it can't be opened (no `/dev/uinput` permission),
    // carry on without hotkey support instead of refusing to start.
    let mut keyboard = match Keyboard::open() {
        Ok(keyboard) => Some(keyboard),
        Err(err) => {
            eprintln!("soomfon: hotkeys disabled: {err}");
            None
        }
    };

    loop {
        match Deck::connect_first().await {
            Ok(Some(deck)) => {
                if let Err(err) = drive(deck, keyboard.as_mut(), &redraw).await {
                    eprintln!("soomfon: device session ended: {err}");
                }
            }
            Ok(None) => {}
            Err(err) => eprintln!("soomfon: could not connect to a deck: {err}"),
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

/// Render the active page, then dispatch key presses and repaint on demand until
/// the deck errors.
///
/// The loop waits on two things at once: input from the deck, and a redraw
/// request from the UI. `next_events` is cancellation-safe at report boundaries
/// (an unread report stays buffered for the next read), so dropping it to handle
/// a redraw never loses a key press — and redraws only arrive while the user is
/// in the app, not mid-press.
async fn drive(
    deck: Deck,
    mut keyboard: Option<&mut Keyboard>,
    redraw: &RedrawSignal,
) -> Result<(), DeviceError> {
    render_active_page(&deck).await?;

    let reader = deck.reader();
    loop {
        tokio::select! {
            events = reader.next_events() => {
                for event in events? {
                    if let InputEvent::KeyDown(key) = event {
                        dispatch(key, keyboard.as_deref_mut());
                    }
                }
            }
            _ = redraw.wait() => {
                render_active_page(&deck).await?;
            }
        }
    }
}

/// Look up the freshly-loaded config and run the pressed key's action.
///
/// Config is read from disk on each press so edits saved in the UI take effect
/// without restarting the session. The session's virtual keyboard is passed
/// through for hotkey actions.
fn dispatch(key: u8, keyboard: Option<&mut Keyboard>) {
    let config = config::load_config().unwrap_or_default();
    let Some(button) = config.active_page().and_then(|page| page.button(key)) else {
        return;
    };
    if let Err(err) = button.action.run(keyboard) {
        eprintln!("soomfon: key {} action failed: {err}", key + 1);
    }
}

/// Mirror the active page's LCD buttons onto the deck.
async fn render_active_page(deck: &Deck) -> Result<(), DeviceError> {
    let config = config::load_config().unwrap_or_default();
    let page = config.active_page();

    for key in 0..deck.lcd_key_count() {
        match page.and_then(|p| p.button(key)) {
            Some(button) => draw_button(deck, key, button).await?,
            None => deck.clear_key(key).await?,
        }
    }
    deck.flush().await
}

/// Draw a single configured button: its label if it has one, else a solid fill.
async fn draw_button(deck: &Deck, key: u8, button: &Button) -> Result<(), DeviceError> {
    match &button.label {
        Some(label) if !label.is_empty() => {
            deck.set_key_text(key, label, button.text_color, button.color)
                .await
        }
        _ => deck.set_key_color(key, button.color).await,
    }
}
