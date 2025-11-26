mod app_state;
mod commands;
mod database;
mod ocr;
mod ocr_tasks;
mod platform;
mod shortcuts;
mod system_tray;
mod translation;

use app_state::AppState;
#[cfg(not(target_os = "macos"))]
use commands::submit_area_for_ocr;
use commands::{
    capture_and_ocr, capture_area_and_ocr, capture_screen, capture_screen_area, clear_history,
    get_api_key, get_app_config, get_setting, get_translation_history, reload_shortcuts,
    save_api_key, save_app_config, save_setting, save_translation, search_history, set_ocr_result,
    start_area_selection, translate_text,
};
use database::Database;
#[cfg(target_os = "macos")]
use platform::submit_area_for_ocr;
use shortcuts::register_shortcuts;
use std::sync::Mutex;
use system_tray::setup_system_tray;
use tauri::Manager;
use translation::{get_supported_languages, TranslationService};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let db = Database::new(app.handle()).expect("数据库初始化失败");
            let translation_service = TranslationService::OpenAI;

            app.manage(AppState {
                db: Mutex::new(db),
                translation_service: Mutex::new(translation_service),
            });

            register_shortcuts(app.handle());
            setup_system_tray(app.handle())?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            translate_text,
            save_translation,
            get_translation_history,
            search_history,
            clear_history,
            save_setting,
            get_setting,
            save_api_key,
            get_api_key,
            save_app_config,
            get_app_config,
            reload_shortcuts,
            capture_screen,
            capture_screen_area,
            capture_and_ocr,
            capture_area_and_ocr,
            submit_area_for_ocr,
            start_area_selection,
            set_ocr_result,
            get_supported_languages
        ])
        .run(tauri::generate_context!())
        .expect("应用启动失败");
}
