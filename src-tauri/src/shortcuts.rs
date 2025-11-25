use crate::{app_state::AppState, commands::capture_and_ocr, database};
use arboard::Clipboard;
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const PREFILL_EVENT: &str = "prefill-text";

pub fn register_shortcuts(app: &AppHandle) {
    let state = app.state::<AppState>();
    let db = state.db.lock().unwrap();

    let _ = app.global_shortcut().unregister_all();

    let hotkeys = if let Ok(Some(config)) = db.get_app_config() {
        config.hotkeys
    } else {
        database::HotkeyConfig {
            popup_window: "Ctrl+Shift+T".to_string(),
            slide_translation: "Ctrl+Shift+S".to_string(),
            screenshot_translation: "Ctrl+Shift+A".to_string(),
        }
    };

    if !hotkeys.popup_window.is_empty() {
        if let Ok(shortcut) = Shortcut::from_str(&hotkeys.popup_window) {
            let app_handle = app.clone();
            match app.global_shortcut().register(shortcut) {
                Ok(_) => {
                    let _ = app.global_shortcut().on_shortcut(
                        shortcut,
                        move |_app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        },
                    );
                    println!("Registered popup window shortcut: {}", hotkeys.popup_window);
                }
                Err(e) => {
                    eprintln!(
                        "Failed to register popup window shortcut '{}': {}",
                        hotkeys.popup_window, e
                    );
                }
            }
        }
    }

    if !hotkeys.screenshot_translation.is_empty() {
        if let Ok(shortcut) = Shortcut::from_str(&hotkeys.screenshot_translation) {
            let app_handle = app.clone();
            match app.global_shortcut().register(shortcut) {
                Ok(_) => {
                    let _ = app.global_shortcut().on_shortcut(
                        shortcut,
                        move |_app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                let handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    handle_fullscreen_ocr_shortcut(handle).await;
                                });
                            }
                        },
                    );
                    println!(
                        "Registered screenshot translation shortcut: {}",
                        hotkeys.screenshot_translation
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Failed to register screenshot translation shortcut '{}': {}",
                        hotkeys.screenshot_translation, e
                    );
                }
            }
        }
    }

    if !hotkeys.slide_translation.is_empty() {
        if let Ok(shortcut) = Shortcut::from_str(&hotkeys.slide_translation) {
            let app_handle = app.clone();
            match app.global_shortcut().register(shortcut) {
                Ok(_) => {
                    let _ = app.global_shortcut().on_shortcut(
                        shortcut,
                        move |_app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                let handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    handle_slide_translation_shortcut(handle).await;
                                });
                            }
                        },
                    );
                    println!(
                        "Registered slide translation shortcut: {}",
                        hotkeys.slide_translation
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Failed to register slide translation shortcut '{}': {}",
                        hotkeys.slide_translation, e
                    );
                }
            }
        }
    }
}

async fn handle_fullscreen_ocr_shortcut(app_handle: AppHandle) {
    let _ = show_main_window(&app_handle);

    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("ocr-pending", true);
    }

    let result = {
        let state = app_handle.state::<AppState>();
        capture_and_ocr(state).await
    };

    if let Some(window) = app_handle.get_webview_window("main") {
        match result {
            Ok(text) => {
                let _ = window.emit("ocr-result", text);
            }
            Err(err) => {
                let _ = window.emit("ocr-result", format!("Error: {}", err));
            }
        }
    }
}

async fn handle_slide_translation_shortcut(app_handle: AppHandle) {
    let _ = show_main_window(&app_handle);

    let selected_text = capture_selected_text();

    if let Some(text) = selected_text {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.emit(PREFILL_EVENT, text);
        }
    }
}

fn show_main_window(app_handle: &AppHandle) -> Option<WebviewWindow> {
    app_handle.get_webview_window("main").map(|window| {
        let _ = window.show();
        let _ = window.set_focus();
        window
    })
}

fn capture_selected_text() -> Option<String> {
    trigger_copy_shortcut();
    std::thread::sleep(std::time::Duration::from_millis(120));

    let mut clipboard = Clipboard::new().ok()?;
    let text = clipboard.get_text().ok()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(target_os = "macos")]
fn trigger_copy_shortcut() {
    use std::process::Command;

    let script = r#"tell application "System Events" to keystroke "c" using {command down}"#;
    let _ = Command::new("osascript").arg("-e").arg(script).output();
}

#[cfg(target_os = "windows")]
fn trigger_copy_shortcut() {
    use std::process::Command;

    let script = r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('^c'); Start-Sleep -Milliseconds 50"#;
    let _ = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output();
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn trigger_copy_shortcut() {}
