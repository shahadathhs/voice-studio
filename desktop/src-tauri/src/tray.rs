//! The icon in the menu bar or system tray, with its menu.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

use crate::dictation;

const SHOW: &str = "show";
const DICTATE: &str = "dictate";
const QUIT: &str = "quit";

/// Adds the tray icon: open the window, start or stop dictation, quit.
pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, SHOW, "Open Voice Studio", true, None::<&str>)?,
            &MenuItem::with_id(app, DICTATE, "Start or stop dictation", true, None::<&str>)?,
            &MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?,
        ],
    )?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("Voice Studio")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            SHOW => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            DICTATE => dictation::toggle(app),
            QUIT => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
