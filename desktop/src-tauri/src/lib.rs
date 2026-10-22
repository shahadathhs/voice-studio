//! Voice Studio desktop shell. A thin layer: the UI calls these commands, which call `vtt-core`
//! in-process. There is no server and no sidecar process.

mod commands;
mod downloads;
mod jobs;
mod library;
mod playback;

/// Starts the desktop application.
///
/// # Errors
/// Returns an error if the window or webview cannot be created.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            jobs::start(app.handle());
            playback::init(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_models,
            commands::install_model,
            commands::cancel_install,
            commands::remove_model,
            jobs::list_jobs,
            jobs::add_jobs,
            jobs::cancel_job,
            jobs::remove_job,
            jobs::clear_finished_jobs,
            jobs::move_job,
            jobs::set_queue_paused,
            library::list_library,
            library::search_library,
            library::get_transcript,
            library::delete_library_item,
            library::render_export,
            library::save_export,
            playback::prepare_playback
        ])
        .run(tauri::generate_context!())
}
