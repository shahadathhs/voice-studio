//! Voice Studio desktop shell. A thin layer: the UI calls these commands, which call `vtt-core`
//! in-process. There is no server and no sidecar process.

mod assistant;
mod clones;
mod commands;
mod dictation;
mod downloads;
mod jobs;
mod library;
mod playback;
mod recording;
mod settings;
mod speech;
mod tray;
mod voices;

/// Starts the desktop application.
///
/// # Errors
/// Returns an error if the window or webview cannot be created.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            jobs::start(app.handle());
            playback::init(app.handle());
            tray::init(app.handle())?;
            dictation::register(app.handle());
            Ok(())
        })
        .invoke_handler(handler())
        .run(tauri::generate_context!())
}

/// Every command the UI can call. Kept out of `run` so that function stays within the line limit.
fn handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        assistant::cancel_text_task,
        assistant::delete_note,
        assistant::list_notes,
        assistant::start_text_task,
        assistant::text_languages,
        commands::app_info,
        dictation::toggle_dictation,
        commands::third_party_notices,
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
        recording::cancel_recording,
        recording::discard_recording,
        recording::list_input_devices,
        recording::save_recording_as,
        recording::start_recording,
        recording::stop_recording,
        settings::get_settings,
        settings::save_settings,
        speech::list_voices,
        speech::start_speech,
        speech::cancel_speech,
        speech::save_speech,
        speech::read_text_file,
        voices::create_voice,
        voices::delete_voice,
        voices::preview_voice,
        clones::clone_voice,
        clones::delete_clone
    ]
}
