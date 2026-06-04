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

/// Build and run the desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_status])
        .run(tauri::generate_context!())
        .expect("error while running the soomfonLinux application");
}
