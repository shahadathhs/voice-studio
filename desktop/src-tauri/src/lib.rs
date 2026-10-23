//! Voice Studio desktop shell. A thin layer: the UI calls these commands, which call `vtt-core`
//! in-process. There is no server and no sidecar process.

mod assistant;
mod commands;
mod downloads;
mod jobs;
mod library;
mod playback;
mod settings;
mod speech;

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
            assistant::cancel_text_task,
            assistant::delete_note,
            assistant::list_notes,
            assistant::start_text_task,
            assistant::text_languages,
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
            library::rename_speaker,
            library::set_line_text,
            library::set_line_speaker,
            library::render_export,
            library::save_export,
            playback::prepare_playback,
            playback::get_waveform,
            settings::get_settings,
            settings::save_settings,
            speech::list_voices,
            speech::start_speech,
            speech::cancel_speech,
            speech::save_speech,
            speech::read_text_file
        ])
        .run(tauri::generate_context!())
}
