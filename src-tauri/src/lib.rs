//! Tauri backend for soomfonLinux.
//!
//! Keep this layer thin: it should only expose [`soomfon_core`] to the
//! frontend through Tauri commands and own the window lifecycle. Application
//! logic belongs in `soomfon-core`, hardware logic in `soomfon-device`.

/// Return a snapshot of the current application state to the frontend.
#[tauri::command]
async fn get_status() -> soomfon_core::Status {
    soomfon_core::status().await
}

/// Load the persisted user configuration (profiles, pages, buttons).
#[tauri::command]
fn get_config() -> Result<soomfon_core::Config, String> {
    soomfon_core::config::load_config().map_err(|e| e.to_string())
}

/// Persist the user configuration sent back by the frontend, then ask the
/// running session to repaint the deck so edits show up without a reconnect.
#[tauri::command]
fn save_config(
    config: soomfon_core::Config,
    redraw: tauri::State<'_, soomfon_core::RedrawSignal>,
) -> Result<(), String> {
    soomfon_core::config::save_config(&config).map_err(|e| e.to_string())?;
    redraw.trigger();
    Ok(())
}

/// Build and run the desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Shared between the background session (which repaints on request) and the
    // `save_config` command (which fires the request after writing config).
    let redraw = soomfon_core::RedrawSignal::new();
    let session_redraw = redraw.clone();

    tauri::Builder::default()
        .manage(redraw)
        .setup(move |_app| {
            // Drive a connected deck in the background: draw the active page and
            // run each key's action on press. The session reconnects on its own.
            tauri::async_runtime::spawn(soomfon_core::run_device_session(session_redraw));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running the soomfonLinux application");
}
