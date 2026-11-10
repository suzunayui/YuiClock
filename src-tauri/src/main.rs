#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fonts;
mod obs;

use std::{fs, path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, State, WebviewUrl, WebviewWindowBuilder, Wry,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_window_state::StateFlags;

const MENU_SHOW: &str = "show";
const MENU_ON_TOP: &str = "on-top";
const MENU_SETTINGS: &str = "settings";
const MENU_QUIT: &str = "quit";
const MAIN_WINDOW: &str = "main";
const SETTINGS_WINDOW: &str = "settings";
const SETTINGS_EVENT: &str = "settings-changed";

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Settings {
    text_color: String,
    /// 0 = opaque, 100 = invisible.
    text_transparency: u32,
    background_color: String,
    /// 0 = opaque, 100 = invisible.
    background_transparency: u32,
    font_family: String,
    bold: bool,
    show_date: bool,
    /// "short-ja" (日), "long-ja" (日曜日), "long-en" (Sunday) or "short-en" (Sun).
    weekday_format: String,
    show_seconds: bool,
    /// Size the date line so it is exactly as wide as the time line.
    align_widths: bool,
    /// 0 = square corners, 100 = fully rounded ends (radius is half the shorter side).
    corner_radius: u32,
    always_on_top: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            text_color: "#ffffff".into(),
            text_transparency: 0,
            background_color: "#000000".into(),
            background_transparency: 0,
            // A monospace font each system ships with.
            font_family: if cfg!(target_os = "macos") { "Menlo" } else { "Consolas" }.into(),
            bold: false,
            show_date: true,
            weekday_format: "short-ja".into(),
            show_seconds: true,
            align_widths: true,
            corner_radius: 0,
            always_on_top: true,
        }
    }
}

struct SettingsStore(Mutex<Settings>);

/// The tray menu lives for the whole run, so its check mark must be kept in sync by hand.
struct TrayOnTopItem(CheckMenuItem<Wry>);

fn settings_path(app: &AppHandle) -> tauri::Result<PathBuf> {
    Ok(app.path().app_config_dir()?.join("settings.json"))
}

fn load_settings(app: &AppHandle) -> Settings {
    settings_path(app)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Apply the settings that live on the native window rather than in the page.
fn apply_window_settings(app: &AppHandle, settings: &Settings) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.set_always_on_top(settings.always_on_top);
    }
    if let Some(item) = app.try_state::<TrayOnTopItem>() {
        let _ = item.0.set_checked(settings.always_on_top);
    }
}

fn store_settings(app: &AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(app).map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    apply_window_settings(app, &settings);
    if let Err(error) = obs::write_settings(app, &settings) {
        eprintln!("Failed to update the OBS page's settings: {error}");
    }
    *app.state::<SettingsStore>().0.lock().unwrap() = settings.clone();
    app.emit(SETTINGS_EVENT, settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(store: State<SettingsStore>) -> Settings {
    store.0.lock().unwrap().clone()
}

#[tauri::command]
fn default_settings() -> Settings {
    Settings::default()
}

#[tauri::command]
fn set_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    store_settings(&app, settings)
}

/// Resize a window's content area to `height` CSS pixels, keeping its width.
#[tauri::command]
fn fit_window_height(window: tauri::WebviewWindow, height: f64) -> tauri::Result<()> {
    let scale = window.scale_factor()?;
    let width = window.inner_size()?.to_logical::<f64>(scale).width;
    window.set_size(tauri::LogicalSize::new(width, height.ceil()))?;
    // It opened centred at a guessed height; centre it again now that the height is known.
    window.center()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ObsInfo {
    url: String,
    /// The whole clock's size for its OBS source, shaped like the desktop clock.
    width: u32,
    height: u32,
    error: Option<String>,
}

/// Whether the OBS page could be written at startup, so the settings window can explain a failure.
struct ObsPage(Option<String>);

#[tauri::command]
fn obs_info(app: AppHandle, page: State<ObsPage>) -> ObsInfo {
    // Size the source like the desktop clock, at a width that stays sharp on a 1080p canvas.
    let width = 800u32;
    let height = app
        .get_webview_window(MAIN_WINDOW)
        .and_then(|window| window.inner_size().ok())
        .filter(|size| size.width > 0 && size.height > 0)
        .map(|size| (f64::from(width) * f64::from(size.height) / f64::from(size.width)).round() as u32)
        .unwrap_or(260)
        .clamp(80, 1600);
    let (url, error) = match (obs::page_url(&app), &page.0) {
        (Ok(url), None) => (url, None),
        (Ok(url), Some(error)) => (url, Some(error.clone())),
        (Err(error), _) => (String::new(), Some(error)),
    };
    ObsInfo {
        url,
        width,
        height,
        error: error.map(|e| format!("OBS用のファイルを書き出せませんでした（{e}）。")),
    }
}

#[tauri::command]
fn list_fonts() -> Vec<fonts::FontFamily> {
    fonts::system_fonts()
}

/// The menu shared by the clock's right-click and the tray icon. Both use the same item ids,
/// so one `on_menu_event` handler serves both.
fn build_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    on_top: bool,
    with_show: bool,
) -> tauri::Result<(Menu<R>, CheckMenuItem<R>)> {
    let menu = Menu::new(manager)?;
    if with_show {
        menu.append(&MenuItem::with_id(manager, MENU_SHOW, "時計を表示", true, None::<&str>)?)?;
    }
    let on_top_item =
        CheckMenuItem::with_id(manager, MENU_ON_TOP, "最前面に表示", true, on_top, None::<&str>)?;
    menu.append(&on_top_item)?;
    menu.append(&MenuItem::with_id(manager, MENU_SETTINGS, "設定…", true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(manager)?)?;
    menu.append(&MenuItem::with_id(manager, MENU_QUIT, "終了…", true, None::<&str>)?)?;
    Ok((menu, on_top_item))
}

#[tauri::command]
fn show_menu(window: tauri::Window, store: State<SettingsStore>) -> tauri::Result<()> {
    let on_top = store.0.lock().unwrap().always_on_top;
    let (menu, _) = build_menu(&window, on_top, false)?;
    window.popup_menu(&menu)
}

/// Bring the clock to the front, and back on screen if it has been dragged off every monitor.
fn show_clock(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    let _ = window.unminimize();
    let _ = window.show();
    let on_screen = match (window.outer_position(), window.outer_size(), window.available_monitors()) {
        (Ok(pos), Ok(size), Ok(monitors)) => monitors.iter().any(|m| {
            let (mp, ms) = (m.position(), m.size());
            pos.x < mp.x + ms.width as i32
                && pos.x + size.width as i32 > mp.x
                && pos.y < mp.y + ms.height as i32
                && pos.y + size.height as i32 > mp.y
        }),
        _ => true,
    };
    if !on_screen {
        let _ = window.center();
    }
    let _ = window.set_focus();
}

fn toggle_on_top(app: &AppHandle) {
    let mut settings = app.state::<SettingsStore>().0.lock().unwrap().clone();
    settings.always_on_top = !settings.always_on_top;
    let _ = store_settings(app, settings);
}

fn open_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, SETTINGS_WINDOW, WebviewUrl::App("settings.html".into()))
        .title("YuiClock 設定")
        // The page sets the final height to fit its content once it has loaded.
        .inner_size(1100.0, 440.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        // The clock may be topmost, so the settings window must be too or it can end up underneath.
        .always_on_top(true)
        // Nothing is dropped onto this window, and Tauri's drop handling could get in the way
        // of dragging the OBS link out of it.
        .disable_drag_drop_handler()
        .center()
        .build();
}

fn confirm_quit(app: &AppHandle) {
    let handle = app.clone();
    let mut dialog = app
        .dialog()
        .message("YuiClock を終了しますか？")
        .title("YuiClock")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "終了".into(),
            "キャンセル".into(),
        ));
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        dialog = dialog.parent(&window);
    }
    dialog.show(move |confirmed| {
        if confirmed {
            handle.exit(0);
        }
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::POSITION | StateFlags::SIZE)
                .with_denylist(&[SETTINGS_WINDOW])
                .build(),
        )
        .setup(|app| {
            // Like `skipTaskbar` on Windows: no Dock icon, the menu bar icon is the way in.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let settings = load_settings(app.handle());
            let (tray_menu, tray_on_top) = build_menu(app, settings.always_on_top, true)?;
            let mut tray = TrayIconBuilder::with_id("main")
                .tooltip("YuiClock")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_clock(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;

            app.manage(TrayOnTopItem(tray_on_top));
            apply_window_settings(app.handle(), &settings);
            app.manage(ObsPage(obs::write_page(app.handle(), &settings).err()));
            app.manage(SettingsStore(Mutex::new(settings)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_menu,
            get_settings,
            default_settings,
            set_settings,
            list_fonts,
            fit_window_height,
            obs_info
        ])
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => show_clock(app),
            MENU_ON_TOP => toggle_on_top(app),
            MENU_SETTINGS => open_settings(app),
            MENU_QUIT => confirm_quit(app),
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("failed to run YuiClock");
}
