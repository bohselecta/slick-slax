mod drives;
mod install;
mod model;

use model::{InstallRequest, IsoInfo, UsbDrive};

#[tauri::command]
fn list_usb_drives() -> Result<Vec<UsbDrive>, String> {
    drives::list_usb_drives()
}

#[tauri::command]
fn choose_iso() -> Result<Option<IsoInfo>, String> {
    let Some(path) = rfd::FileDialog::new().add_filter("Slax ISO image", &["iso"]).pick_file() else { return Ok(None) };
    let metadata = path.metadata().map_err(|error| error.to_string())?;
    let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or("slax.iso").to_string();
    let lower = filename.to_ascii_lowercase();
    let edition = if lower.contains("debian") { Some("Debian edition".into()) } else if lower.contains("slackware") { Some("Slackware edition".into()) } else { None };
    let architecture = if lower.contains("64") || lower.contains("x64") { Some("64-bit".into()) } else if lower.contains("32") || lower.contains("x86") { Some("32-bit".into()) } else { None };
    Ok(Some(IsoInfo { path: path.to_string_lossy().to_string(), filename, size_bytes: metadata.len(), slax_root_found: false, edition, architecture }))
}

#[tauri::command]
fn open_official_download() -> Result<(), String> {
    open::that("https://www.slax.org/#getslax").map_err(|error| error.to_string())
}

#[tauri::command]
async fn install_slax(app: tauri::AppHandle, request: InstallRequest) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || install::install(app, request)).await.map_err(|error| error.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_usb_drives, choose_iso, open_official_download, install_slax])
        .run(tauri::generate_context!())
        .expect("error while running SlickSlax");
}

