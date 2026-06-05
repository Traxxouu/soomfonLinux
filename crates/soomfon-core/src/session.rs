//! The device session: connect to a deck, mirror the active page onto it, and
//! run each key's action when it is pressed.
//!
//! This is the runtime that ties the persisted [`config`](crate::config) to the
//! hardware. The desktop backend spawns [`run_device_session`] once at startup
//! and the session looks after connecting, drawing and dispatching on its own.

use std::time::Duration;

use soomfon_device::{Deck, DeviceError, InputEvent};

use crate::config::{self, Button};

/// How long to wait before retrying after a disconnect or a connection error.
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

/// Run the device session forever: (re)connect to a deck and dispatch input.
///
/// This never returns. When no deck is present, or the connection drops, it
/// waits [`RECONNECT_DELAY`] and tries again, so plugging the device in (or back
/// in) is enough to start driving it.
pub async fn run_device_session() {
    loop {
        match Deck::connect_first().await {
            Ok(Some(deck)) => {
                if let Err(err) = drive(deck).await {
                    eprintln!("soomfon: device session ended: {err}");
                }
            }
            Ok(None) => {}
            Err(err) => eprintln!("soomfon: could not connect to a deck: {err}"),
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

/// Render the active page once, then dispatch key presses until the deck errors.
async fn drive(deck: Deck) -> Result<(), DeviceError> {
    render_active_page(&deck).await?;

    let reader = deck.reader();
    loop {
        for event in reader.next_events().await? {
            if let InputEvent::KeyDown(key) = event {
                dispatch(key);
            }
        }
    }
}

/// Look up the freshly-loaded config and run the pressed key's action.
///
/// Config is read from disk on each press so edits saved in the UI take effect
/// without restarting the session.
fn dispatch(key: u8) {
    let config = config::load_config().unwrap_or_default();
    let Some(button) = config.active_page().and_then(|page| page.button(key)) else {
        return;
    };
    if let Err(err) = button.action.run() {
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
