//! Voice Studio desktop shell. A thin layer: the UI calls these commands, which call `vtt-core`
//! in-process. There is no server and no sidecar process.

mod commands;
mod downloads;

/// Starts the desktop application.
///
/// # Errors
/// Returns an error if the window or webview cannot be created.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_models,
            commands::install_model,
            commands::cancel_install,
            commands::remove_model
        ])
        .run(tauri::generate_context!())
}
