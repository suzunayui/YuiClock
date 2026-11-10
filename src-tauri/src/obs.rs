//! The clock as a page for an OBS browser source.
//!
//! The page and the current settings are written as files under the app's config folder, and
//! OBS opens them through its `http://absolute/<path>` scheme, the same one it uses for a
//! browser source's local file. So OBS can show the clock even when YuiClock isn't running:
//! the page re-reads `settings.json` every second and simply keeps the last settings written.

use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};

use crate::Settings;

const PAGE_FILES: &[(&str, &str)] = &[
    ("index.html", include_str!("../../ui/obs.html")),
    ("style.css", include_str!("../../ui/style.css")),
    ("format.js", include_str!("../../ui/format.js")),
    ("clock.js", include_str!("../../ui/clock.js")),
    ("obs.js", include_str!("../../ui/obs.js")),
];

fn obs_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("obs"))
        .map_err(|e| e.to_string())
}

/// Write the page (refreshing it after an update) and the current settings.
pub fn write_page(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let dir = obs_dir(app)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (name, body) in PAGE_FILES {
        fs::write(dir.join(name), body).map_err(|e| e.to_string())?;
    }
    write_settings(app, settings)
}

/// Replace `settings.json` in one step, so the page never reads a half-written file.
pub fn write_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let dir = obs_dir(app)?;
    let json = serde_json::to_vec(settings).map_err(|e| e.to_string())?;
    let partial = dir.join("settings.json.partial");
    fs::write(&partial, json).map_err(|e| e.to_string())?;
    fs::rename(&partial, dir.join("settings.json")).map_err(|e| e.to_string())
}

/// The page's address for OBS, e.g. `http://absolute/C:/Users/.../obs/index.html`.
pub fn page_url(app: &AppHandle) -> Result<String, String> {
    let path = obs_dir(app)?.join("index.html");
    Ok(absolute_url(&path.to_string_lossy()))
}

fn absolute_url(path: &str) -> String {
    let mut url = String::from("http://absolute/");
    for byte in path.replace('\\', "/").bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                url.push(byte as char)
            }
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }
    url
}

#[cfg(test)]
mod tests {
    use super::absolute_url;

    #[test]
    fn turns_a_windows_path_into_an_obs_url() {
        assert_eq!(
            absolute_url(r"C:\Users\mikan\AppData\Roaming\com.suzunayui.yuiclock\obs\index.html"),
            "http://absolute/C:/Users/mikan/AppData/Roaming/com.suzunayui.yuiclock/obs/index.html"
        );
    }

    #[test]
    fn keeps_the_leading_slash_of_a_mac_path() {
        // OBS itself builds "http://absolute/" + path, so a Unix path keeps both slashes.
        assert_eq!(
            absolute_url("/Users/yui/Library/Application Support/com.suzunayui.yuiclock/obs/index.html"),
            "http://absolute//Users/yui/Library/Application%20Support/com.suzunayui.yuiclock/obs/index.html"
        );
    }

    #[test]
    fn escapes_spaces_and_non_ascii() {
        assert_eq!(absolute_url(r"C:\Users\ゆい さん\x.html"), "http://absolute/C:/Users/%E3%82%86%E3%81%84%20%E3%81%95%E3%82%93/x.html");
    }
}
