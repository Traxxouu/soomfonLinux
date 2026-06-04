//! Detect a connected deck and print every key / encoder event.
//!
//! This is the manual hardware check for the device layer. Plug the deck in,
//! install the udev rule (`packaging/udev/99-soomfon.rules`), then run:
//!
//! ```sh
//! cargo run -p soomfon-device --example monitor
//! ```

use soomfon_device::{list_devices, Deck};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devices = list_devices().await?;

    if devices.is_empty() {
        println!("No supported Soomfon device found.");
        println!("Is it plugged in, and is the udev rule installed?");
        return Ok(());
    }

    println!("Detected {} device(s):", devices.len());
    for device in &devices {
        let serial = device
            .serial
            .as_deref()
            .map(|s| format!(", serial {s}"))
            .unwrap_or_default();
        println!(
            "  - {} (VID {:04x} PID {:04x}, {} keys, {} encoders{})",
            device.model, device.vid, device.pid, device.keys, device.encoders, serial,
        );
    }

    let Some(deck) = Deck::connect_first().await? else {
        println!("Device disappeared before we could connect.");
        return Ok(());
    };

    println!(
        "\nConnected to {}. Press keys / turn knobs (Ctrl-C to quit).\n",
        deck.model()
    );

    let reader = deck.reader();
    loop {
        for event in reader.next_events().await? {
            println!("{event:?}");
        }
    }
}
