//! Draw a colour + label to each LCD key, as a hardware check for rendering.
//!
//! Plug the deck in, install the udev rule
//! (`packaging/udev/70-soomfon.rules`), then run:
//!
//! ```sh
//! cargo run -p soomfon-device --example render
//! ```
//!
//! Each of the six screen keys lights up a different colour with its number.
//! The images persist on the panel after the program exits.

use soomfon_device::Deck;

/// One colour per LCD key. Text is drawn in near-black for contrast.
const PALETTE: [[u8; 3]; 6] = [
    [0xE6, 0x39, 0x46], // red
    [0xF7, 0x7F, 0x00], // orange
    [0xF1, 0xC4, 0x0F], // yellow
    [0x2A, 0x9D, 0x8F], // teal
    [0x45, 0x7B, 0x9D], // blue
    [0x9B, 0x5D, 0xE5], // violet
];

const INK: [u8; 3] = [0x14, 0x14, 0x14];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(deck) = Deck::connect_first().await? else {
        println!("No supported Soomfon device found.");
        println!("Is it plugged in, and is the udev rule installed?");
        return Ok(());
    };

    println!("Connected to {}.", deck.model());
    deck.set_brightness(80).await?;
    deck.clear_all_keys().await?;

    for key in 0..deck.lcd_key_count() {
        let color = PALETTE[key as usize % PALETTE.len()];
        deck.set_key_text(key, &(key + 1).to_string(), INK, color)
            .await?;
    }

    deck.flush().await?;
    println!(
        "Drew {} labelled keys. Look at the panel — the images stay after exit.",
        deck.lcd_key_count()
    );

    Ok(())
}
