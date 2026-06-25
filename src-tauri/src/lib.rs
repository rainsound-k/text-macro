use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

// ── macOS focus tracking ──────────────────────────────────────────────────────
// The picker window steals key focus while it's open. To paste into the app the
// user was actually working in, we remember which app was frontmost right before
// the picker opens, then reactivate it just before simulating ⌘V.
#[cfg(target_os = "macos")]
mod macos_focus {
    use std::sync::atomic::{AtomicI32, Ordering};
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    static PREV_PID: AtomicI32 = AtomicI32::new(-1);

    /// Capture the PID of the currently frontmost application.
    pub fn remember_frontmost() {
        unsafe {
            let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
            if workspace.is_null() {
                return;
            }
            let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
            if app.is_null() {
                return;
            }
            let pid: i32 = msg_send![app, processIdentifier];
            PREV_PID.store(pid, Ordering::SeqCst);
        }
    }

    /// Bring the previously remembered app back to the front.
    pub fn restore_frontmost() {
        let pid = PREV_PID.load(Ordering::SeqCst);
        if pid < 0 {
            return;
        }
        unsafe {
            let app: *mut AnyObject = msg_send![
                class!(NSRunningApplication),
                runningApplicationWithProcessIdentifier: pid
            ];
            if app.is_null() {
                return;
            }
            // NSApplicationActivateAllWindows (1) | NSApplicationActivateIgnoringOtherApps (2)
            let _: bool = msg_send![app, activateWithOptions: 3usize];
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Macro {
    id: String,
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppSettings {
    hotkey: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Space".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExportData {
    version: u32,
    macros: Vec<Macro>,
}

struct AppState {
    macros: Mutex<Vec<Macro>>,
    settings: Mutex<AppSettings>,
    data_dir: std::path::PathBuf,
}

impl AppState {
    fn macros_path(&self) -> std::path::PathBuf {
        self.data_dir.join("macros.json")
    }

    fn settings_path(&self) -> std::path::PathBuf {
        self.data_dir.join("settings.json")
    }

    fn flush_macros(&self, macros: &[Macro]) {
        if let Ok(json) = serde_json::to_string_pretty(macros) {
            let _ = std::fs::write(self.macros_path(), json);
        }
    }

    fn flush_settings(&self, settings: &AppSettings) {
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = std::fs::write(self.settings_path(), json);
        }
    }
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
fn get_macros(state: State<AppState>) -> Vec<Macro> {
    state.macros.lock().unwrap().clone()
}

#[tauri::command]
fn add_macro(state: State<AppState>, title: String, content: String) -> Macro {
    let item = Macro {
        id: Uuid::new_v4().to_string(),
        title,
        content,
    };
    let mut macros = state.macros.lock().unwrap();
    macros.push(item.clone());
    state.flush_macros(&macros);
    item
}

#[tauri::command]
fn update_macro(state: State<AppState>, id: String, title: String, content: String) -> Result<(), String> {
    let mut macros = state.macros.lock().unwrap();
    let idx = macros.iter().position(|m| m.id == id).ok_or("not found")?;
    macros[idx].title = title;
    macros[idx].content = content;
    state.flush_macros(&macros);
    Ok(())
}

#[tauri::command]
fn delete_macro(state: State<AppState>, id: String) -> Result<(), String> {
    let mut macros = state.macros.lock().unwrap();
    let before = macros.len();
    macros.retain(|m| m.id != id);
    if macros.len() < before {
        state.flush_macros(&macros);
        Ok(())
    } else {
        Err("not found".to_string())
    }
}

#[tauri::command]
fn reorder_macros(state: State<AppState>, ids: Vec<String>) -> Result<(), String> {
    let mut macros = state.macros.lock().unwrap();
    let reordered: Vec<Macro> = ids
        .iter()
        .filter_map(|id| macros.iter().find(|m| &m.id == id).cloned())
        .collect();
    if reordered.len() != macros.len() {
        return Err("invalid ids".to_string());
    }
    *macros = reordered;
    state.flush_macros(&macros);
    Ok(())
}

#[tauri::command]
fn export_macros(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let macros = state.macros.lock().unwrap().clone();
    let data = ExportData { version: 1, macros };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;

    let path = app
        .dialog()
        .file()
        .set_title("매크로 내보내기")
        .set_file_name("macros.json")
        .blocking_save_file();

    if let Some(fp) = path {
        let dest = fp.into_path().map_err(|e| e.to_string())?;
        std::fs::write(dest, json).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn import_macros(app: AppHandle, state: State<AppState>) -> Result<Vec<Macro>, String> {
    let path = app
        .dialog()
        .file()
        .set_title("매크로 가져오기")
        .add_filter("JSON", &["json"])
        .blocking_pick_file();

    let Some(fp) = path else {
        return Ok(state.macros.lock().unwrap().clone());
    };

    let src = fp.into_path().map_err(|e| e.to_string())?;
    let json = std::fs::read_to_string(src).map_err(|e| e.to_string())?;
    let data: ExportData = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let mut macros = state.macros.lock().unwrap();
    for imported in data.macros {
        match macros.iter().position(|m| m.id == imported.id) {
            Some(idx) => macros[idx] = imported,
            None => macros.push(imported),
        }
    }
    state.flush_macros(&macros);
    Ok(macros.clone())
}

#[tauri::command]
fn paste_text(content: String, app: AppHandle) -> Result<(), String> {
    // Hide picker immediately; the rest runs in a background thread
    if let Some(w) = app.get_webview_window("picker") {
        let _ = w.hide();
    }

    let app2 = app.clone();
    std::thread::spawn(move || {
        // Let the picker window finish hiding.
        std::thread::sleep(std::time::Duration::from_millis(80));

        // Phase 1: reactivate the previously focused app and load the clipboard.
        let _ = app.run_on_main_thread(move || {
            #[cfg(target_os = "macos")]
            macos_focus::restore_frontmost();
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(&content);
            }
        });

        // Give macOS time to bring the target app fully to the front before typing.
        std::thread::sleep(std::time::Duration::from_millis(150));

        // Phase 2: simulate the paste shortcut into the now-focused app.
        let _ = app2.run_on_main_thread(move || {
            #[cfg(target_os = "macos")]
            {
                use core_graphics::event::{
                    CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode,
                };
                use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

                // Post ⌘V directly as keycode 9 (kVK_ANSI_V) + Command flag, rather
                // than going through a Unicode event which macOS may not treat as the
                // paste key equivalent.
                const KVK_ANSI_V: CGKeyCode = 9;
                if let Ok(source) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
                    if let Ok(down) =
                        CGEvent::new_keyboard_event(source.clone(), KVK_ANSI_V, true)
                    {
                        down.set_flags(CGEventFlags::CGEventFlagCommand);
                        down.post(CGEventTapLocation::HID);
                    }
                    if let Ok(up) = CGEvent::new_keyboard_event(source, KVK_ANSI_V, false) {
                        up.set_flags(CGEventFlags::CGEventFlagCommand);
                        up.post(CGEventTapLocation::HID);
                    }
                }
            }
            #[cfg(target_os = "windows")]
            {
                use enigo::{Direction, Enigo, Key, Keyboard, Settings};
                if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                    let _ = enigo.key(Key::Control, Direction::Press);
                    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                    let _ = enigo.key(Key::Control, Direction::Release);
                }
            }
        });
    });

    Ok(())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn update_hotkey(app: AppHandle, state: State<AppState>, hotkey: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let new_shortcut = parse_shortcut_str(&hotkey)?;

    // Unregister the old shortcut before registering the new one
    let current = state.settings.lock().unwrap().hotkey.clone();
    if let Ok(old) = parse_shortcut_str(&current) {
        let _ = app.global_shortcut().unregister(old);
    }

    app.global_shortcut()
        .register(new_shortcut)
        .map_err(|e| e.to_string())?;

    let mut settings = state.settings.lock().unwrap();
    settings.hotkey = hotkey;
    state.flush_settings(&settings);
    Ok(())
}

#[tauri::command]
fn get_autostart(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    if enabled {
        app.autolaunch().enable().map_err(|e| e.to_string())
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn show_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    { true }
}

#[tauri::command]
fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }
}

// ── Shortcut parsing ────────────────────────────────────────────────────────

fn parse_shortcut_str(s: &str) -> Result<tauri_plugin_global_shortcut::Shortcut, String> {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

    let mut mods = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in s.split('+') {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" | "option"   => mods |= Modifiers::ALT,
            "shift"            => mods |= Modifiers::SHIFT,
            "super" | "meta" | "cmd" | "command" => mods |= Modifiers::META,
            key => {
                key_code = Some(str_to_code(key)
                    .ok_or_else(|| format!("알 수 없는 키: {key}"))?);
            }
        }
    }

    let code = key_code.ok_or_else(|| "키가 지정되지 않았습니다".to_string())?;
    Ok(Shortcut::new(if mods.is_empty() { None } else { Some(mods) }, code))
}

fn str_to_code(s: &str) -> Option<tauri_plugin_global_shortcut::Code> {
    use tauri_plugin_global_shortcut::Code;
    Some(match s {
        "space"     => Code::Space,
        "enter" | "return" => Code::Enter,
        "tab"       => Code::Tab,
        "escape" | "esc" => Code::Escape,
        "backspace" => Code::Backspace,
        "delete" | "del" => Code::Delete,
        "home"      => Code::Home,
        "end"       => Code::End,
        "pageup"    => Code::PageUp,
        "pagedown"  => Code::PageDown,
        "arrowup"   => Code::ArrowUp,
        "arrowdown" => Code::ArrowDown,
        "arrowleft" => Code::ArrowLeft,
        "arrowright"=> Code::ArrowRight,
        "f1"  => Code::F1,  "f2"  => Code::F2,  "f3"  => Code::F3,
        "f4"  => Code::F4,  "f5"  => Code::F5,  "f6"  => Code::F6,
        "f7"  => Code::F7,  "f8"  => Code::F8,  "f9"  => Code::F9,
        "f10" => Code::F10, "f11" => Code::F11, "f12" => Code::F12,
        "a" => Code::KeyA, "b" => Code::KeyB, "c" => Code::KeyC,
        "d" => Code::KeyD, "e" => Code::KeyE, "f" => Code::KeyF,
        "g" => Code::KeyG, "h" => Code::KeyH, "i" => Code::KeyI,
        "j" => Code::KeyJ, "k" => Code::KeyK, "l" => Code::KeyL,
        "m" => Code::KeyM, "n" => Code::KeyN, "o" => Code::KeyO,
        "p" => Code::KeyP, "q" => Code::KeyQ, "r" => Code::KeyR,
        "s" => Code::KeyS, "t" => Code::KeyT, "u" => Code::KeyU,
        "v" => Code::KeyV, "w" => Code::KeyW, "x" => Code::KeyX,
        "y" => Code::KeyY, "z" => Code::KeyZ,
        "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2,
        "3" => Code::Digit3, "4" => Code::Digit4, "5" => Code::Digit5,
        "6" => Code::Digit6, "7" => Code::Digit7, "8" => Code::Digit8,
        "9" => Code::Digit9,
        _ => return None,
    })
}

// ── Helpers ─────────────────────────────────────────────────────────────────

// Center the picker on whichever monitor the cursor is currently on.
fn center_on_cursor_screen(picker: &tauri::WebviewWindow) {
    let Ok(monitors) = picker.available_monitors() else { return; };
    if monitors.is_empty() { return; }

    let idx = if let Ok(cursor) = picker.cursor_position() {
        monitors.iter().position(|m| {
            let p = m.position();
            let s = m.size();
            cursor.x >= p.x as f64
                && cursor.x < (p.x as f64 + s.width as f64)
                && cursor.y >= p.y as f64
                && cursor.y < (p.y as f64 + s.height as f64)
        })
    } else {
        None
    }
    .unwrap_or(0);

    let m = &monitors[idx];
    let pos = m.position();
    let size = m.size();
    let scale = m.scale_factor();

    // Convert physical monitor rect to logical, then center the 540×420 window.
    let x = pos.x as f64 / scale + (size.width as f64 / scale - 540.0) / 2.0;
    let y = pos.y as f64 / scale + (size.height as f64 / scale - 420.0) / 2.0;
    let _ = picker.set_position(tauri::LogicalPosition::new(x, y));
}

// ── App setup ───────────────────────────────────────────────────────────────

fn default_macros() -> Vec<Macro> {
    vec![
        Macro { id: Uuid::new_v4().to_string(), title: "안녕하세요".into(), content: "안녕하세요! 무엇을 도와드릴까요?".into() },
        Macro { id: Uuid::new_v4().to_string(), title: "감사합니다".into(), content: "감사합니다! 좋은 하루 되세요.".into() },
        Macro { id: Uuid::new_v4().to_string(), title: "확인했습니다".into(), content: "확인했습니다. 바로 처리해드리겠습니다.".into() },
        Macro { id: Uuid::new_v4().to_string(), title: "잠시만요".into(), content: "잠시만요, 확인 후 답변 드리겠습니다.".into() },
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        if let Some(picker) = app.get_webview_window("picker") {
                            if picker.is_visible().unwrap_or(false) {
                                let _ = picker.hide();
                            } else {
                                #[cfg(target_os = "macos")]
                                macos_focus::remember_frontmost();
                                center_on_cursor_screen(&picker);
                                let _ = picker.show();
                                let _ = picker.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let macros_path = data_dir.join("macros.json");
            let settings_path = data_dir.join("settings.json");

            let macros: Vec<Macro> = std::fs::read_to_string(&macros_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(default_macros);

            let settings: AppSettings = std::fs::read_to_string(&settings_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            // Persist defaults on first run
            if !macros_path.exists() {
                if let Ok(json) = serde_json::to_string_pretty(&macros) {
                    let _ = std::fs::write(&macros_path, json);
                }
            }
            if !settings_path.exists() {
                if let Ok(json) = serde_json::to_string_pretty(&settings) {
                    let _ = std::fs::write(&settings_path, json);
                }
            }

            app.manage(AppState {
                macros: Mutex::new(macros),
                settings: Mutex::new(settings.clone()),
                data_dir: data_dir.clone(),
            });

            // macOS: hide dock icon
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Register global shortcut from saved settings (falls back to Alt+Space)
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
            let shortcut = parse_shortcut_str(&settings.hotkey)
                .unwrap_or_else(|_| Shortcut::new(Some(Modifiers::ALT), Code::Space));
            app.global_shortcut().register(shortcut)?;

            // System tray
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

            let settings_item = MenuItem::with_id(app, "settings", "설정 열기", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("TextMacro")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(w) = app.get_webview_window("settings") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(picker) = app.get_webview_window("picker") {
                            if picker.is_visible().unwrap_or(false) {
                                let _ = picker.hide();
                            } else {
                                #[cfg(target_os = "macos")]
                                macos_focus::remember_frontmost();
                                center_on_cursor_screen(&picker);
                                let _ = picker.show();
                                let _ = picker.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_macros,
            add_macro,
            update_macro,
            delete_macro,
            reorder_macros,
            export_macros,
            import_macros,
            paste_text,
            get_settings,
            update_hotkey,
            get_autostart,
            set_autostart,
            show_settings,
            check_accessibility,
            open_accessibility_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
