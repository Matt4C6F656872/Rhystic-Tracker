use std::path::{Path, PathBuf};
use serde::Serialize;
use tauri::Manager;
use crate::db::{self, DatabaseManager};
use crate::card_db;
use crate::settings;
use crate::tailer;
use crate::theme::{get_mana_theme, ManaTheme};
use crate::commands::collection::{card_img_cache_dir, clear_universe_cache};

#[derive(serde::Serialize)]
pub struct AppEnvironmentInfo {
    environment: String,
    is_test: bool,
    db_name: String,
}

#[tauri::command]
pub fn get_app_environment() -> AppEnvironmentInfo {
    let env = db::DatabaseManager::resolve_env();
    let is_prod = env.to_lowercase() == "production";
    AppEnvironmentInfo {
        environment: if is_prod { "production".to_string() } else { "development".to_string() },
        is_test: !is_prod,
        db_name: if is_prod { "rhystic.db".to_string() } else { "rhystic_dev.db".to_string() },
    }
}

#[tauri::command]
pub fn get_active_theme(theme_id: String) -> ManaTheme {
    get_mana_theme(&theme_id)
}

#[derive(serde::Serialize)]
pub struct MemoryStats {
    main_kb: u64,
    webkit_kb: u64,
    network_kb: u64,
    total_kb: u64,
}

fn read_rss_kb(pid: i32) -> Option<u64> {
    let status = std::fs::read_to_string(format!("/proc/{}/status", pid)).ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb: u64 = rest.trim().split_whitespace().next()?.parse().ok()?;
            return Some(kb);
        }
    }
    None
}

fn child_rss_with_name(name_substring: &str) -> u64 {
    let self_pid = std::process::id() as i32;
    let Ok(entries) = std::fs::read_dir("/proc") else { return 0 };
    for entry in entries.flatten() {
        let pid_str = entry.file_name().to_string_lossy().to_string();
        let Ok(pid) = pid_str.parse::<i32>() else { continue };
        if pid == self_pid { continue; }
        // Confirm it is a direct child of this process
        let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", pid)) else { continue };
        let is_child = status.lines().any(|l| l.starts_with("PPid:") && l.trim_end().ends_with(&self_pid.to_string()));
        if !is_child { continue; }
        let Ok(cmdline) = std::fs::read(format!("/proc/{}/cmdline", pid)) else { continue };
        let cmd = String::from_utf8_lossy(&cmdline);
        if cmd.contains(name_substring) {
            return read_rss_kb(pid).unwrap_or(0);
        }
    }
    0
}

/// Read the RSS of the main process and its WebKit child processes (Linux).
/// Dev/test diagnostic only — the frontend gates this behind isTestEnv.
#[tauri::command]
#[cfg(target_os = "linux")]
pub fn get_memory_stats() -> MemoryStats {
    let main_kb = read_rss_kb(std::process::id() as i32).unwrap_or(0);
    let webkit_kb = child_rss_with_name("WebKitWebProcess");
    let network_kb = child_rss_with_name("WebKitNetworkProcess");
    MemoryStats {
        main_kb,
        webkit_kb,
        network_kb,
        total_kb: main_kb + webkit_kb + network_kb,
    }
}

#[tauri::command]
#[cfg(not(target_os = "linux"))]
pub fn get_memory_stats() -> MemoryStats {
    MemoryStats { main_kb: 0, webkit_kb: 0, network_kb: 0, total_kb: 0 }
}

// Resolve the effective MTGA log path: stored override > RHYSTIC_MTGA_LOG >
// auto-discovery. Returns an empty string when none can be found.
pub fn resolve_effective_log_path() -> String {
    let settings = settings::load_settings();
    if let Some(p) = settings.mtga_log_path.as_deref() {
        if !p.is_empty() {
            return p.to_string();
        }
    }
    if let Ok(p) = std::env::var("RHYSTIC_MTGA_LOG") {
        if !p.is_empty() {
            return p;
        }
    }
    tailer::discover_log_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Shared runtime state so the frontend can query the active log path and have
/// the tailer restart when the user changes it.
pub struct LogPathState {
    pub path_tx: tokio::sync::watch::Sender<String>,
}

#[tauri::command]
pub fn get_log_path(state: tauri::State<LogPathState>) -> String {
    state.path_tx.borrow().clone()
}

#[tauri::command]
pub fn set_log_path(state: tauri::State<LogPathState>, path: String) -> Result<String, String> {
    let trimmed = path.trim().to_string();
    let mut settings = settings::load_settings();
    settings.mtga_log_path = if trimmed.is_empty() { None } else { Some(trimmed.clone()) };
    settings::save_settings(&settings)?;
    let effective = resolve_effective_log_path();
    state.path_tx.send(effective.clone()).map_err(|_| "log path channel closed".to_string())?;
    Ok(effective)
}

#[tauri::command]
pub fn get_minimize_to_tray() -> bool {
    settings::load_settings().minimize_to_tray
}

#[tauri::command]
pub fn set_minimize_to_tray(enabled: bool) -> Result<bool, String> {
    let mut settings = settings::load_settings();
    settings.minimize_to_tray = enabled;
    settings::save_settings(&settings)?;
    Ok(enabled)
}


#[derive(serde::Serialize)]
pub struct CacheStats {
    size_bytes: u64,
    file_count: usize,
}

#[tauri::command]
pub fn get_cache_stats(app: tauri::AppHandle) -> Result<CacheStats, String> {
    let dir = card_img_cache_dir(&app)?;
    let mut total_size = 0u64;
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_size += meta.len();
                    count += 1;
                }
            }
        }
    }
    Ok(CacheStats { size_bytes: total_size, file_count: count })
}

#[tauri::command]
pub fn clear_image_cache(app: tauri::AppHandle) -> Result<CacheStats, String> {
    let dir = card_img_cache_dir(&app)?;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    Ok(CacheStats { size_bytes: 0, file_count: 0 })
}

/// Directory where MTGA avatar bust images are cached locally on demand.
/// Lives under Tauri's appConfigDir (~/.config/rhystic-tracker/avatars)
/// covered by Tauri assetProtocol scope.
fn avatar_cache_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let avatar_dir = dir.join("avatars");
    std::fs::create_dir_all(&avatar_dir).map_err(|e| e.to_string())?;
    Ok(avatar_dir)
}

fn avatar_filename(avatar_id: &str) -> String {
    let mut s: String = avatar_id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    s.truncate(80);
    format!("{}.png", s)
}

#[tauri::command]
pub fn get_avatar_cache_stats(app: tauri::AppHandle) -> Result<CacheStats, String> {
    let dir = avatar_cache_dir(&app)?;
    let mut total_size = 0u64;
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_size += meta.len();
                    count += 1;
                }
            }
        }
    }
    Ok(CacheStats { size_bytes: total_size, file_count: count })
}

#[tauri::command]
pub fn clear_avatar_cache(app: tauri::AppHandle) -> Result<CacheStats, String> {
    let dir = avatar_cache_dir(&app)?;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    Ok(CacheStats { size_bytes: 0, file_count: 0 })
}

#[derive(Serialize)]
pub struct AvatarExtractResult {
    success: bool,
    count: u32,
    message: String,
}

#[derive(Serialize)]
pub struct AvatarSyncStatus {
    new_avatars_available: bool,
    client_detected: bool,
    client_bundle_mtime: u64,
    last_synced_mtime: u64,
    cached_avatar_count: u32,
}

/// Discovers the MTGA Downloads directory containing AssetBundle and ALT folders.
fn find_mtga_downloads_dir() -> Option<PathBuf> {
    // 1. Try from raw card db
    if let Some(raw) = crate::card_db::find_latest_raw_card_db() {
        if let Some(parent) = raw.parent() {
            if parent.file_name().and_then(|s| s.to_str()) == Some("Raw") {
                if let Some(dl) = parent.parent() {
                    if dl.join("AssetBundle").exists() || dl.join("ALT").exists() {
                        return Some(dl.to_path_buf());
                    }
                }
            } else if parent.join("AssetBundle").exists() || parent.join("ALT").exists() {
                return Some(parent.to_path_buf());
            }
        }
    }

    // 2. Try from settings / log path
    let settings = crate::settings::load_settings();
    let log_path = settings.mtga_log_path
        .map(PathBuf::from)
        .or_else(|| crate::tailer::discover_log_path());

    if let Some(lp) = log_path {
        let mut curr = lp.parent();
        for _ in 0..8 {
            if let Some(c) = curr {
                let cand = c.join("MTGA_Data/Downloads");
                if cand.exists() {
                    return Some(cand);
                }
                if c.file_name().and_then(|s| s.to_str()) == Some("drive_c") {
                    for sub in [
                        "Program Files/Wizards of the Coast/MTGA/MTGA_Data/Downloads",
                        "Program Files (x86)/Wizards of the Coast/MTGA/MTGA_Data/Downloads",
                        "Program Files/MTGA/MTGA_Data/Downloads",
                        "Program Files (x86)/MTGA/MTGA_Data/Downloads",
                        "MTGA/MTGA_Data/Downloads",
                        "Games/MTGA/MTGA_Data/Downloads",
                    ] {
                        let sc = c.join(sub);
                        if sc.exists() {
                            return Some(sc);
                        }
                    }
                }
                curr = c.parent();
            } else {
                break;
            }
        }
    }

    // 3. Scan common paths
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".steam/steam/steamapps/common/MTGA/MTGA_Data/Downloads"),
            home.join(".steam/root/steamapps/common/MTGA/MTGA_Data/Downloads"),
            home.join(".local/share/Steam/steamapps/common/MTGA/MTGA_Data/Downloads"),
            home.join(".var/app/com.valvesoftware.Steam/.steam/steam/steamapps/common/MTGA/MTGA_Data/Downloads"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/MTGA/MTGA_Data/Downloads"),
            home.join("Games/magic-the-gathering-arena/drive_c/Program Files/Wizards of the Coast/MTGA/MTGA_Data/Downloads"),
            home.join("Games/mtga/drive_c/Program Files/Wizards of the Coast/MTGA/MTGA_Data/Downloads"),
            home.join("Games/Magic-The-Gathering-Arena/drive_c/Program Files/Wizards of the Coast/MTGA/MTGA_Data/Downloads"),
            home.join(".local/share/lutris/runners/wine/mtga/drive_c/Program Files/Wizards of the Coast/MTGA/MTGA_Data/Downloads"),
            home.join(".wine/drive_c/Program Files/Wizards of the Coast/MTGA/MTGA_Data/Downloads"),
            home.join(".wine/drive_c/Program Files (x86)/Wizards of the Coast/MTGA/MTGA_Data/Downloads"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }

    None
}

/// Finds the newest modification time among avatar asset bundles and ALT files.
fn get_latest_client_avatar_mtime(downloads_dir: &Path) -> u64 {
    let mut latest_mtime = 0u64;

    // Check ALT directory
    let alt_dir = downloads_dir.join("ALT");
    if let Ok(entries) = std::fs::read_dir(&alt_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("ALT_Avatar_") && name.ends_with(".mtga") {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                latest_mtime = latest_mtime.max(duration.as_secs());
                            }
                        }
                    }
                }
            }
        }
    }

    // Check AssetBundle directory for BustPayload
    let bundle_dir = downloads_dir.join("AssetBundle");
    if let Ok(entries) = std::fs::read_dir(&bundle_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("Bucket_Avatar.BustPayload_") && name.ends_with(".mtga") {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                latest_mtime = latest_mtime.max(duration.as_secs());
                            }
                        }
                    }
                }
            }
        }
    }

    latest_mtime
}

#[tauri::command]
pub async fn check_new_avatars_status(app: tauri::AppHandle) -> Result<AvatarSyncStatus, String> {
    let out_dir = avatar_cache_dir(&app)?;
    let cached_avatar_count = match std::fs::read_dir(&out_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("png"))
            .count() as u32,
        Err(_) => 0,
    };

    let downloads_opt = find_mtga_downloads_dir();
    let client_detected = downloads_opt.is_some();

    let client_bundle_mtime = if let Some(ref dl) = downloads_opt {
        get_latest_client_avatar_mtime(dl)
    } else {
        0
    };

    let settings = crate::settings::load_settings();
    let last_synced_mtime = settings.last_avatar_sync_bundle_mtime;

    // New avatars are available if:
    // 1. Client has avatar bundles, AND
    // 2. Either cached avatar count is 0 (first sync needed), OR
    //    client bundles have a newer mtime than what was previously synced.
    let new_avatars_available = client_detected
        && client_bundle_mtime > 0
        && (cached_avatar_count == 0 || client_bundle_mtime > last_synced_mtime);

    Ok(AvatarSyncStatus {
        new_avatars_available,
        client_detected,
        client_bundle_mtime,
        last_synced_mtime,
        cached_avatar_count,
    })
}

/// Ensures a working Python environment with UnityPy and Pillow is available.
/// Returns the path to the python binary.
fn resolve_or_bootstrap_python(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 1. Check dedicated rhystic-tracker data directory venv: ~/.local/share/rhystic-tracker/venv/bin/python3
    let mut candidate_pythons = Vec::new();

    if let Some(data_dir) = dirs::data_dir() {
        candidate_pythons.push(data_dir.join("rhystic-tracker/venv/bin/python3"));
        candidate_pythons.push(data_dir.join("rhystic-tracker/venv/bin/python"));
    }

    // 2. Check app config dir venv
    if let Ok(cfg_dir) = app.path().app_config_dir() {
        candidate_pythons.push(cfg_dir.join("venv/bin/python3"));
        candidate_pythons.push(cfg_dir.join("venv/bin/python"));
    }

    // 3. Check legacy /tmp/unity_env/bin/python3
    candidate_pythons.push(PathBuf::from("/tmp/unity_env/bin/python3"));

    // 4. System pythons
    candidate_pythons.push(PathBuf::from("python3"));
    candidate_pythons.push(PathBuf::from("python"));

    // Check if any existing candidate already has UnityPy and PIL installed
    for py in &candidate_pythons {
        let test_output = std::process::Command::new(py)
            .args(["-c", "import UnityPy, PIL; print('OK')"])
            .output();

        if let Ok(out) = test_output {
            if out.status.success() {
                return Ok(py.clone());
            }
        }
    }

    // If none has UnityPy installed, bootstrap a persistent virtual environment in data_dir
    let venv_dir = dirs::data_dir()
        .map(|d| d.join("rhystic-tracker/venv"))
        .or_else(|| app.path().app_config_dir().ok().map(|d| d.join("venv")))
        .unwrap_or_else(|| std::env::temp_dir().join("rhystic-tracker-venv"));

    let venv_python = if cfg!(windows) {
        venv_dir.join("Scripts/python.exe")
    } else {
        venv_dir.join("bin/python3")
    };

    let venv_pip = if cfg!(windows) {
        venv_dir.join("Scripts/pip.exe")
    } else {
        venv_dir.join("bin/pip")
    };

    // If the venv doesn't exist, create it
    if !venv_python.exists() {
        if let Some(parent) = venv_dir.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let create_status = std::process::Command::new("python3")
            .args(["-m", "venv", &venv_dir.to_string_lossy()])
            .output();

        if let Err(e) = create_status {
            return Err(format!("Failed to create Python virtual environment via python3: {}", e));
        }
    }

    // Install UnityPy and Pillow
    if venv_pip.exists() {
        let install_status = std::process::Command::new(&venv_pip)
            .args(["install", "UnityPy", "Pillow"])
            .output();

        if let Ok(out) = install_status {
            if !out.status.success() {
                let err = String::from_utf8_lossy(&out.stderr);
                return Err(format!("pip install UnityPy Pillow failed: {}", err));
            }
        } else if let Err(e) = install_status {
            return Err(format!("Failed to run pip install: {}", e));
        }
    }

    // Verify newly bootstrapped python
    let verify = std::process::Command::new(&venv_python)
        .args(["-c", "import UnityPy, PIL; print('OK')"])
        .output();

    if let Ok(out) = verify {
        if out.status.success() {
            return Ok(venv_python);
        }
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("Bootstrapped python failed verification: {}", err));
    }

    Err("Could not configure a valid Python environment with UnityPy and Pillow.".to_string())
}

/// Directory where custom background images uploaded by the user are stored.
fn background_cache_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let bg_dir = dir.join("backgrounds");
    std::fs::create_dir_all(&bg_dir).map_err(|e| e.to_string())?;
    Ok(bg_dir)
}

#[tauri::command]
pub fn save_custom_background(app: tauri::AppHandle, filename: String, data: Vec<u8>) -> Result<String, String> {
    let dir = background_cache_dir(&app)?;
    // Sanitize filename to avoid directory traversal
    let safe_name: String = filename.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let safe_name = if safe_name.is_empty() { "background.webp".to_string() } else { safe_name };
    let path = dir.join(&safe_name);
    std::fs::write(&path, &data).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

const EMBEDDED_AVATAR_EXTRACTOR_SCRIPT: &str = include_str!("../../scripts/extract_mtga_avatars.py");

#[tauri::command]
pub async fn extract_avatars_from_mtga_client(app: tauri::AppHandle) -> Result<AvatarExtractResult, String> {
    let out_dir = avatar_cache_dir(&app)?;
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    // Derive active MTGA raw directory or log path
    let raw_dir = crate::card_db::find_latest_raw_card_db()
        .and_then(|f| f.parent().map(|p| p.to_path_buf()));
    let raw_arg = raw_dir.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

    let settings = crate::settings::load_settings();
    let log_path = settings.mtga_log_path
        .map(PathBuf::from)
        .or_else(|| crate::tailer::discover_log_path());
    let log_arg = log_path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

    // Try locating extractor script on disk first
    let mut script_candidates = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            script_candidates.push(parent.join("scripts/extract_mtga_avatars.py"));
            script_candidates.push(parent.join("../../scripts/extract_mtga_avatars.py"));
        }
    }

    if let Ok(res_dir) = app.path().resource_dir() {
        script_candidates.push(res_dir.join("scripts/extract_mtga_avatars.py"));
    }

    // Also check standard app config directory
    if let Ok(cfg_dir) = app.path().app_config_dir() {
        script_candidates.push(cfg_dir.join("scripts/extract_mtga_avatars.py"));
    }

    let script = if let Some(found) = script_candidates.into_iter().find(|p| p.exists()) {
        found
    } else {
        // Materialize the embedded script to app config dir or temp dir
        let target_script_dir = match app.path().app_config_dir() {
            Ok(cfg_dir) => cfg_dir.join("scripts"),
            Err(_) => std::env::temp_dir().join("rhystic-tracker-scripts"),
        };
        let _ = std::fs::create_dir_all(&target_script_dir);
        let embedded_path = target_script_dir.join("extract_mtga_avatars.py");
        std::fs::write(&embedded_path, EMBEDDED_AVATAR_EXTRACTOR_SCRIPT)
            .map_err(|e| format!("Failed to materialize embedded avatar extractor script: {}", e))?;
        embedded_path
    };

    let py_bin = match resolve_or_bootstrap_python(&app) {
        Ok(p) => p,
        Err(bootstrap_err) => {
            return Ok(AvatarExtractResult {
                success: false,
                count: 0,
                message: format!("Python environment error: {}. Please ensure Python 3 is installed.", bootstrap_err),
            });
        }
    };

    let output = std::process::Command::new(&py_bin)
        .arg(&script)
        .arg(&raw_arg)
        .arg(&out_dir)
        .arg(&log_arg)
        .output();

    let mut last_error = String::new();
    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let combined = format!("{} {}", stdout.trim(), stderr.trim());
                last_error = combined.trim().to_string();
            }
        }
        Err(e) => {
            last_error = format!("Failed to launch {:?}: {}", py_bin, e);
        }
    }

    // Ensure default adventurer / NPE avatar fallback exists in local cache
    let def_avatar = out_dir.join("default_adventurer.png");
    if !def_avatar.exists() {
        // If an NPE image was extracted, copy it over
        let npe_cand = out_dir.join("Avatar_Basic_NPE.png");
        if npe_cand.exists() {
            let _ = std::fs::copy(&npe_cand, &def_avatar);
        }
    }

    let count = match std::fs::read_dir(&out_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("png"))
            .count() as u32,
        Err(_) => 0,
    };

    if count == 0 && !last_error.is_empty() {
        return Ok(AvatarExtractResult {
            success: false,
            count: 0,
            message: format!("Avatar extraction failed: {}", last_error),
        });
    }

    // Update settings with latest synced bundle mtime if MTGA downloads dir is found
    if let Some(dl) = find_mtga_downloads_dir() {
        let latest_mtime = get_latest_client_avatar_mtime(&dl);
        if latest_mtime > 0 {
            let mut updated_settings = crate::settings::load_settings();
            updated_settings.last_avatar_sync_bundle_mtime = latest_mtime;
            let _ = crate::settings::save_settings(&updated_settings);
        }
    }

    Ok(AvatarExtractResult {
        success: true,
        count,
        message: format!("Successfully synced {} avatars in local storage.", count),
    })
}

#[tauri::command]
pub fn save_avatar_image(app: tauri::AppHandle, avatar_id: String, data: Vec<u8>) -> Result<String, String> {
    let dir = avatar_cache_dir(&app)?;
    let path = dir.join(avatar_filename(&avatar_id));
    std::fs::write(&path, &data).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn has_avatar_image(app: tauri::AppHandle, avatar_id: String) -> Result<Option<String>, String> {
    let dir = avatar_cache_dir(&app)?;
    let clean = avatar_id.trim();
    if clean.is_empty() {
        let def = dir.join("default_adventurer.png");
        if def.exists() {
            return Ok(Some(def.to_string_lossy().to_string()));
        }
        return Ok(None);
    }

    // Generate candidates
    let mut candidates = vec![avatar_filename(clean)];

    // If Avatar_Basic_Character_SET
    let stripped_prefix = clean
        .strip_prefix("Avatar_Basic_")
        .or_else(|| clean.strip_prefix("Avatar_Portrait_"))
        .or_else(|| clean.strip_prefix("Avatar_Standard_"))
        .or_else(|| clean.strip_prefix("Avatar_"))
        .unwrap_or(clean);

    candidates.push(format!("{}.png", stripped_prefix));

    if let Some((char_name, set_code)) = stripped_prefix.rsplit_once('_') {
        candidates.push(format!("{}_{}.png", set_code, char_name));
        candidates.push(format!("{}.png", char_name));
    }

    for c in &candidates {
        let p = dir.join(c);
        if p.exists() {
            return Ok(Some(p.to_string_lossy().to_string()));
        }
    }

    // Try case-insensitive / substring lookup
    if let Ok(entries) = std::fs::read_dir(&dir) {
        let lower_clean = clean.to_lowercase();
        let lower_stripped = stripped_prefix.to_lowercase();
        let mut best_match: Option<PathBuf> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(fname) = path.file_stem().and_then(|s| s.to_str()) {
                let f_lower = fname.to_lowercase();
                if f_lower == lower_clean || f_lower == lower_stripped {
                    return Ok(Some(path.to_string_lossy().to_string()));
                }
                if f_lower.contains(&lower_stripped) || lower_stripped.contains(&f_lower) {
                    best_match = Some(path.clone());
                }
            }
        }
        if let Some(p) = best_match {
            return Ok(Some(p.to_string_lossy().to_string()));
        }
    }

    // Guaranteed fallback to default_adventurer if present
    let def = dir.join("default_adventurer.png");
    if def.exists() {
        return Ok(Some(def.to_string_lossy().to_string()));
    }

    Ok(None)
}

#[derive(serde::Serialize)]
pub struct DatabaseStats {
    db_filename: String,
    db_path: String,
    size_bytes: u64,
    match_count: i64,
}

#[tauri::command]
pub async fn get_database_stats() -> Result<DatabaseStats, String> {
    let db = DatabaseManager::init().await.map_err(|e| e.to_string())?;
    let match_count = db.get_match_count().await.unwrap_or(0);
    let db_filename = db.db_filename.clone();

    let env_mode = db::DatabaseManager::resolve_env();
    let is_prod = env_mode.to_lowercase() == "production";
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rhystic-tracker");
    let actual_path = config_dir.join(if is_prod { "rhystic.db" } else { "rhystic_dev.db" });

    let size_bytes = std::fs::metadata(&actual_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(DatabaseStats {
        db_filename,
        db_path: actual_path.to_string_lossy().to_string(),
        size_bytes,
        match_count,
    })
}

#[derive(serde::Serialize)]
pub struct SetupStatus {
    setup_completed: bool,
    card_count: i64,
    log_path: Option<String>,
    raw_path: Option<String>,
}

#[tauri::command]
pub async fn get_setup_status() -> Result<SetupStatus, String> {
    let settings = settings::load_settings();
    let db = DatabaseManager::init().await.map_err(|e| e.to_string())?;
    let card_count: i64 = sqlx::query_scalar("SELECT count(*) FROM cards_cache")
        .fetch_one(db.pool())
        .await
        .unwrap_or(0);

    let log_path = settings.mtga_log_path.clone().or_else(|| {
        tailer::discover_log_path().map(|p| p.to_string_lossy().to_string())
    });

    let raw_path = card_db::find_latest_raw_card_db().map(|p| p.to_string_lossy().to_string());

    Ok(SetupStatus {
        setup_completed: settings.setup_completed && card_count > 0,
        card_count,
        log_path,
        raw_path,
    })
}

#[tauri::command]
pub async fn complete_setup() -> Result<(), String> {
    let mut settings = settings::load_settings();
    settings.setup_completed = true;
    settings::save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_setup_wizard() -> Result<(), String> {
    let mut settings = settings::load_settings();
    settings.setup_completed = false;
    settings::save_settings(&settings).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct SyncCardDbResult {
    success: bool,
    card_count: usize,
    elapsed_ms: u128,
    raw_path: Option<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn sync_card_database() -> Result<SyncCardDbResult, String> {
    let db = DatabaseManager::init().await.map_err(|e| e.to_string())?;
    let raw_path = card_db::find_latest_raw_card_db().map(|p| p.to_string_lossy().to_string());

    match card_db::sync_card_cache(db.pool()).await {
        Ok((count, elapsed_ms)) => {
            clear_universe_cache();
            Ok(SyncCardDbResult {
                success: true,
                card_count: count,
                elapsed_ms,
                raw_path,
                error: None,
            })
        }
        Err(e) => {
            Ok(SyncCardDbResult {
                success: false,
                card_count: 0,
                elapsed_ms: 0,
                raw_path,
                error: Some(e.to_string()),
            })
        }
    }
}

#[tauri::command]
pub async fn get_raw_card_db_status() -> Result<serde_json::Value, String> {
    let db = DatabaseManager::init().await.map_err(|e| e.to_string())?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM cards_cache")
        .fetch_one(db.pool())
        .await
        .unwrap_or(0);
    let raw_path = card_db::find_latest_raw_card_db().map(|p| p.to_string_lossy().to_string());
    Ok(serde_json::json!({
        "card_count": count,
        "raw_path": raw_path,
    }))
}

#[tauri::command]
pub async fn set_raw_path(path: String) -> Result<String, String> {
    let mut settings = settings::load_settings();
    let trimmed = path.trim().to_string();
    settings.mtga_raw_dir = if trimmed.is_empty() { None } else { Some(trimmed.clone()) };
    settings::save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(trimmed)
}

#[tauri::command]
pub async fn export_database_backup(destination_path: String) -> Result<String, String> {
    let env_mode = db::DatabaseManager::resolve_env();
    let is_prod = env_mode.to_lowercase() == "production";
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rhystic-tracker");
    let src_path = config_dir.join(if is_prod { "rhystic.db" } else { "rhystic_dev.db" });

    if !src_path.exists() {
        return Err("Source database file does not exist".to_string());
    }

    let dest = std::path::PathBuf::from(&destination_path);
    std::fs::copy(&src_path, &dest).map_err(|e| format!("Failed to copy database: {}", e))?;
    Ok(format!("Database backup successfully created at {:?}", destination_path))
}


/// Sets the window always-on-top state.
#[tauri::command]
pub async fn set_always_on_top(window: tauri::Window, enabled: bool) -> Result<(), String> {
    window.set_always_on_top(enabled).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_sync_settings_status() -> Result<serde_json::Value, String> {
    let db = DatabaseManager::init().await.map_err(|e| e.to_string())?;
    let log_path = resolve_effective_log_path();
    crate::sync::ensure_verified_identity_from_log(&log_path);
    let cfg = crate::sync::load_sync_config();
    let stats = crate::sync::get_outbox_stats(&db).await;
    Ok(serde_json::json!({
        "enabled": cfg.enabled,
        "contributor_id": cfg.contributor_id,
        "server_url": cfg.server_url,
        "include_deck_cards": cfg.include_deck_cards,
        "share_full_decklist": cfg.share_full_decklist,
        "share_player_identity": cfg.share_player_identity,
        "verified_arena_handle": cfg.verified_arena_handle,
        "arena_account_hash": cfg.arena_account_hash,
        "sync_key": cfg.sync_key,
        "pending_count": stats.pending_count,
        "synced_count": stats.synced_count,
        "failed_count": stats.failed_count,
        "last_synced_at": stats.last_synced_at,
    }))
}

#[tauri::command]
pub async fn update_sync_settings(
    enabled: bool,
    server_url: Option<String>,
    include_deck_cards: Option<bool>,
    share_full_decklist: Option<bool>,
    share_player_identity: Option<bool>,
    sync_key: Option<String>,
    rotate_contributor_id: Option<bool>,
    custom_contributor_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let log_path = resolve_effective_log_path();
    crate::sync::ensure_verified_identity_from_log(&log_path);
    let mut cfg = crate::sync::load_sync_config();
    cfg.enabled = enabled;
    if let Some(url) = server_url {
        let trimmed = url.trim().to_string();
        if !trimmed.is_empty() {
            cfg.server_url = trimmed;
        }
    }
    if let Some(inc) = include_deck_cards {
        cfg.include_deck_cards = inc;
    }
    if let Some(full) = share_full_decklist {
        cfg.share_full_decklist = full;
    }
    if let Some(ident) = share_player_identity {
        cfg.share_player_identity = ident;
    }
    if let Some(key) = sync_key {
        let trimmed = key.trim().to_string();
        cfg.sync_key = if trimmed.is_empty() { None } else { Some(trimmed) };
    }
    if let Some(custom_id) = custom_contributor_id {
        let trimmed = custom_id.trim();
        if !trimmed.is_empty() {
            if trimmed.len() < 8 || trimmed.len() > 128 {
                return Err("Contributor token must be between 8 and 128 characters.".to_string());
            }
            cfg.contributor_id = trimmed.to_string();
        }
    } else if rotate_contributor_id.unwrap_or(false) {
        cfg.contributor_id = crate::sync::generate_anonymous_uuid();
    }
    crate::sync::save_sync_config(&cfg)?;
    if cfg.enabled {
        let cfg_clone = cfg.clone();
        tokio::spawn(async move {
            if let Some(db) = DatabaseManager::init().await.ok() {
                let _ = crate::sync::refresh_historical_outbox(&db, &cfg_clone).await;
                let _ = crate::sync::flush_outbox_batch(&db, &cfg_clone, 500).await;
            }
        });
    }
    get_sync_settings_status().await
}

#[tauri::command]
pub async fn enqueue_historical_sync() -> Result<serde_json::Value, String> {
    let db = DatabaseManager::init().await.map_err(|e| e.to_string())?;
    let log_path = resolve_effective_log_path();
    crate::sync::ensure_verified_identity_from_log(&log_path);
    let cfg = crate::sync::load_sync_config();
    let queued = crate::sync::refresh_historical_outbox(&db, &cfg).await?;
    let mut flushed_count = 0usize;
    let mut last_error = None;
    if cfg.enabled {
        if let Ok(report) = crate::sync::flush_outbox_batch(&db, &cfg, 500).await {
            flushed_count = report.flushed_count;
            last_error = report.last_error;
        }
    }
    let stats = crate::sync::get_outbox_stats(&db).await;
    Ok(serde_json::json!({
        "queued": queued,
        "flushed_count": flushed_count,
        "last_error": last_error,
        "pending_count": stats.pending_count,
        "synced_count": stats.synced_count,
        "failed_count": stats.failed_count,
        "last_synced_at": stats.last_synced_at,
    }))
}

#[tauri::command]
pub async fn flush_sync_outbox() -> Result<serde_json::Value, String> {
    let db = DatabaseManager::init().await.map_err(|e| e.to_string())?;
    let cfg = crate::sync::load_sync_config();
    let report = crate::sync::flush_outbox_batch(&db, &cfg, 200).await?;
    let stats = crate::sync::get_outbox_stats(&db).await;
    Ok(serde_json::json!({
        "flushed_count": report.flushed_count,
        "last_error": report.last_error,
        "pending_count": stats.pending_count,
        "synced_count": stats.synced_count,
        "failed_count": stats.failed_count,
        "last_synced_at": stats.last_synced_at,
    }))
}


// ===========================================================================
// In-app update checker
// ===========================================================================
//
// Upstream-compatible by design: no fork URL is hard-coded in Rust. The GitHub
// repository whose releases are checked is resolved at *build* time from:
//   1. `RHYSTIC_UPDATE_REPO` (compile-time env var, `owner/repo` or a GitHub URL)
//   2. `repository` in src-tauri/Cargo.toml (exposed as `CARGO_PKG_REPOSITORY`)
//   3. the canonical upstream, `Balthazzahr/Rhystic-Tracker`
// so every fork that sets its own `repository` key gets correct update checks.

/// Canonical upstream repository, used when the build carries no repo metadata.
const DEFAULT_UPDATE_REPO: &str = "Balthazzahr/Rhystic-Tracker";
/// Upper bound for a downloaded release asset (guards against runaway downloads).
const MAX_UPDATE_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

/// Extract `owner/repo` from a GitHub URL (`https://github.com/o/r[.git]`,
/// `git@github.com:o/r.git`, `github.com/o/r`) or, when `allow_bare_slug` is
/// set, from a plain `owner/repo` string. Non-GitHub URLs yield `None`.
fn parse_github_slug(raw: &str, allow_bare_slug: bool) -> Option<String> {
    let s = raw.trim();
    let s = s.split(|c| c == '?' || c == '#').next().unwrap_or("");

    let path = if let Some(rest) = s.strip_prefix("git@github.com:") {
        rest
    } else if let Some((_, after_scheme)) = s.split_once("://") {
        let (authority, path) = after_scheme.split_once('/').unwrap_or((after_scheme, ""));
        let host = authority.rsplit('@').next().unwrap_or(authority);
        if !host.eq_ignore_ascii_case("github.com") && !host.eq_ignore_ascii_case("www.github.com") {
            return None;
        }
        path
    } else if let Some(rest) = s.strip_prefix("github.com/") {
        rest
    } else if allow_bare_slug {
        s
    } else {
        return None;
    };

    let mut parts = path.split('/').filter(|p| !p.is_empty());
    let owner = parts.next()?;
    let repo = parts.next()?;
    let repo = repo.strip_suffix(".git").unwrap_or(repo);
    let is_valid = |p: &str| {
        !p.is_empty()
            && p != "."
            && p != ".."
            && p.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    if is_valid(owner) && is_valid(repo) {
        Some(format!("{}/{}", owner, repo))
    } else {
        None
    }
}

/// GitHub `owner/repo` whose releases this build checks for updates.
pub fn resolve_update_repo_slug() -> String {
    option_env!("RHYSTIC_UPDATE_REPO")
        .and_then(|r| parse_github_slug(r, true))
        .or_else(|| option_env!("CARGO_PKG_REPOSITORY").and_then(|r| parse_github_slug(r, false)))
        .unwrap_or_else(|| DEFAULT_UPDATE_REPO.to_string())
}

/// Minimal semver-ish version. Field order gives the comparison order, and
/// `stable: false < true` ranks `2.1.0-beta` below `2.1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ReleaseVersion {
    major: u64,
    minor: u64,
    patch: u64,
    stable: bool,
}

/// Parse tags like `v2.1.0`, `2.1`, `rhystic-tracker-v2.1.0-beta.1+build5`.
fn parse_release_version(raw: &str) -> Option<ReleaseVersion> {
    let start = raw.find(|c: char| c.is_ascii_digit())?;
    let v = &raw[start..];
    let v = v.split('+').next().unwrap_or(v);
    let (core, pre) = match v.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (v, None),
    };
    let mut nums = core.split('.').map(|p| p.trim().parse::<u64>());
    let major = nums.next()?.ok()?;
    let minor = match nums.next() {
        Some(n) => n.ok()?,
        None => 0,
    };
    let patch = match nums.next() {
        Some(n) => n.ok()?,
        None => 0,
    };
    Some(ReleaseVersion { major, minor, patch, stable: pre.map_or(true, |p| p.is_empty()) })
}

/// True only when both versions parse and `latest` is strictly newer.
fn is_newer_version(latest: &str, current: &str) -> bool {
    match (parse_release_version(latest), parse_release_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

/// `v2.1.0` -> `2.1.0` for display.
fn display_version(tag: &str) -> String {
    tag.find(|c: char| c.is_ascii_digit())
        .map(|i| tag[i..].to_string())
        .unwrap_or_else(|| tag.to_string())
}

#[derive(serde::Deserialize, Debug, Clone)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    html_url: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AppUpdateInfo {
    update_available: bool,
    current_version: String,
    latest_version: String,
    repo_slug: String,
    release_url: String,
    release_notes: Option<String>,
    download_url: Option<String>,
    release_name: Option<String>,
    published_at: Option<String>,
    asset_name: Option<String>,
    /// `self_replace` | `installer` | `release_page`
    install_mode: String,
    install_mode_reason: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AppUpdateInstallResult {
    /// `binary_replaced` | `installer_launched` | `opened_release_page` | `up_to_date`
    action: String,
    message: String,
    latest_version: String,
    restart_required: bool,
    will_exit: bool,
    checksum_verified: bool,
    path: Option<String>,
}

/// How an update can be applied for this platform / install location.
#[allow(dead_code)] // SelfReplace is only constructed on Linux, Installer on Windows/macOS.
enum InstallPlan {
    /// Atomically swap the running binary (user-local Linux installs).
    SelfReplace(PathBuf),
    /// Download and launch the platform installer (Windows setup / macOS dmg).
    Installer,
    /// Fall back to opening the GitHub release page, with the reason why.
    ReleasePage(String),
}

fn update_http_client(timeout: std::time::Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        // GitHub's API rejects requests without a User-Agent.
        .user_agent(concat!("rhystic-tracker/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(timeout)
        .build()
        .map_err(|e| format!("Failed to initialise HTTP client: {}", e))
}

async fn fetch_latest_release(client: &reqwest::Client, repo_slug: &str) -> Result<GithubRelease, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo_slug);
    let resp = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("Could not reach GitHub: {}", e))?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(format!("No published releases found for {}", repo_slug));
    }
    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("GitHub API rate limit reached — please try again later".to_string());
    }
    if !status.is_success() {
        return Err(format!("GitHub API returned HTTP {}", status));
    }
    resp.json::<GithubRelease>()
        .await
        .map_err(|e| format!("Unexpected GitHub release payload: {}", e))
}

/// Rank release assets for the running platform (lower is better).
/// Asset names follow .github/workflows/release.yml.
fn asset_rank(name: &str) -> Option<u8> {
    let n = name.to_ascii_lowercase();
    #[cfg(target_os = "windows")]
    {
        if n.ends_with("setup.exe") {
            return Some(0);
        }
        if n.ends_with(".msi") {
            return Some(1);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if n.ends_with(".dmg") {
            return Some(0);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if n.contains("linux") && n.contains(std::env::consts::ARCH) && n.ends_with(".tar.gz") {
            return Some(0);
        }
    }
    let _ = n;
    None
}

fn select_platform_asset(assets: &[GithubReleaseAsset]) -> Option<&GithubReleaseAsset> {
    assets
        .iter()
        .filter_map(|a| asset_rank(&a.name).map(|rank| (rank, a)))
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, a)| a)
}

/// Resolve the running binary if it is safe to replace in place: a release
/// build, user-writable, and not owned by a package manager / cargo target dir.
#[cfg(target_os = "linux")]
fn linux_self_replace_target() -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        return Err("Development builds are never replaced in place".to_string());
    }
    if std::env::var_os("APPIMAGE").is_some() {
        return Err("AppImage installs must be replaced with the new AppImage".to_string());
    }
    // Canonical path cached by Tauri before main() — unaffected by later swaps.
    let exe = tauri::utils::platform::current_exe()
        .map_err(|e| format!("Cannot resolve the running executable: {}", e))?;
    let exe_str = exe.to_string_lossy().to_string();
    if ["/usr/", "/opt/", "/app/", "/nix/", "/snap/", "/gnu/"]
        .iter()
        .any(|prefix| exe_str.starts_with(prefix))
    {
        return Err(format!("{} is managed by your system package manager", exe_str));
    }
    if exe_str.contains("/target/release/") || exe_str.contains("/target/debug/") {
        return Err("Running directly from a cargo build directory".to_string());
    }
    let parent = exe.parent().ok_or_else(|| "Executable has no parent directory".to_string())?;
    let probe = parent.join(".rhystic-tracker.update-probe");
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let _ = std::fs::remove_file(&probe);
        }
        Err(_) => return Err(format!("{} is not writable", parent.display())),
    }
    Ok(exe)
}

fn plan_install(asset: Option<&GithubReleaseAsset>) -> InstallPlan {
    if asset.is_none() {
        return InstallPlan::ReleasePage("No downloadable release asset matches this platform".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        match linux_self_replace_target() {
            Ok(exe) => InstallPlan::SelfReplace(exe),
            Err(reason) => InstallPlan::ReleasePage(reason),
        }
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        if cfg!(debug_assertions) {
            InstallPlan::ReleasePage("Development builds are not updated automatically".to_string())
        } else {
            InstallPlan::Installer
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        InstallPlan::ReleasePage("Automatic updates are not supported on this platform".to_string())
    }
}

/// Open a GitHub URL in the user's default browser (no shell interpolation).
fn open_external_github_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://github.com/") {
        return Err(format!("Refusing to open non-GitHub URL: {}", url));
    }
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(url);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("rundll32");
        c.arg("url.dll,FileProtocolHandler").arg(url);
        c
    };
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        let mut child = cmd.spawn().map_err(|e| format!("Could not open {}: {}", url, e))?;
        // Reap the launcher so it doesn't linger as a zombie.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(format!("Opening links is not supported on this platform: {}", url))
    }
}

/// Look up the expected SHA-256 for `asset_name` in the release's SHA256SUMS.txt.
/// `Ok(None)` when the release ships no checksum file; `Err` when it does but
/// the asset is missing from it (never install an unlisted download).
async fn expected_asset_sha256(
    client: &reqwest::Client,
    release: &GithubRelease,
    asset_name: &str,
) -> Result<Option<String>, String> {
    let Some(sums) = release.assets.iter().find(|a| {
        a.name.eq_ignore_ascii_case("SHA256SUMS.txt") || a.name.eq_ignore_ascii_case("SHA256SUMS")
    }) else {
        return Ok(None);
    };
    let text = client
        .get(&sums.browser_download_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Failed to download SHA256SUMS.txt: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read SHA256SUMS.txt: {}", e))?;
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        if let (Some(hash), Some(name)) = (fields.next(), fields.next()) {
            if name.trim_start_matches('*') == asset_name {
                return Ok(Some(hash.to_ascii_lowercase()));
            }
        }
    }
    Err(format!(
        "SHA256SUMS.txt does not list {}; refusing to install an unverified download",
        asset_name
    ))
}

/// Stream `url` to `dest`, returning the lowercase hex SHA-256 of the payload.
async fn download_update_asset(client: &reqwest::Client, url: &str, dest: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncWriteExt;

    let mut resp = client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Download failed: {}", e))?;
    if resp.content_length().map_or(false, |len| len > MAX_UPDATE_DOWNLOAD_BYTES) {
        return Err("Release asset is unexpectedly large; aborting".to_string());
    }

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("Cannot create {}: {}", dest.display(), e))?;
    let mut hasher = Sha256::new();
    let mut total: u64 = 0;
    while let Some(chunk) = resp.chunk().await.map_err(|e| format!("Download interrupted: {}", e))? {
        total += chunk.len() as u64;
        if total > MAX_UPDATE_DOWNLOAD_BYTES {
            return Err("Release asset is unexpectedly large; aborting".to_string());
        }
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed writing {}: {}", dest.display(), e))?;
    }
    file.flush().await.map_err(|e| e.to_string())?;
    file.sync_all().await.map_err(|e| e.to_string())?;
    if total == 0 {
        return Err("Downloaded release asset is empty".to_string());
    }
    Ok(hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect())
}

/// Keep only filename-safe characters from a GitHub asset name.
fn sanitize_asset_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    let cleaned = cleaned.trim_start_matches('.').to_string();
    if cleaned.is_empty() {
        "rhystic-tracker-update.bin".to_string()
    } else {
        cleaned
    }
}

#[cfg(target_os = "linux")]
fn find_file_named(dir: &Path, names: &[&std::ffi::OsStr], depth: u32) -> Option<PathBuf> {
    let mut subdirs = Vec::new();
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let Ok(file_type) = entry.file_type() else { continue };
        let path = entry.path();
        if file_type.is_file() && path.file_name().map_or(false, |n| names.contains(&n)) {
            return Some(path);
        }
        if file_type.is_dir() {
            subdirs.push(path);
        }
    }
    if depth == 0 {
        return None;
    }
    subdirs.into_iter().find_map(|d| find_file_named(&d, names, depth - 1))
}

/// Extract the release tarball and atomically swap it over `target_exe`.
/// The running process keeps its (now unlinked) inode; the new binary is
/// picked up on the next launch. User data directories are never touched.
#[cfg(target_os = "linux")]
fn linux_apply_update_tarball(archive: &Path, work_dir: &Path, target_exe: &Path) -> Result<(), String> {
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;

    let extract_dir = work_dir.join("extracted");
    std::fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
    let status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(archive)
        .arg("-C")
        .arg(&extract_dir)
        .arg("--no-same-owner")
        .status()
        .map_err(|e| format!("Failed to run tar: {}", e))?;
    if !status.success() {
        return Err(format!("Extracting the release archive failed ({})", status));
    }

    let exe_name = target_exe
        .file_name()
        .ok_or_else(|| "Executable path has no file name".to_string())?;
    let new_bin = find_file_named(&extract_dir, &[std::ffi::OsStr::new("rhystic-tracker"), exe_name], 4)
        .ok_or_else(|| "Release archive does not contain a rhystic-tracker binary".to_string())?;

    let mut magic = [0u8; 4];
    std::fs::File::open(&new_bin)
        .and_then(|mut f| f.read_exact(&mut magic))
        .map_err(|e| format!("Cannot read new binary: {}", e))?;
    if &magic != b"\x7fELF" {
        return Err("Downloaded file is not a Linux executable".to_string());
    }

    // Stage next to the target so the final rename stays on one filesystem
    // (atomic). install-linux.sh also cleans up this `.update-new` name.
    let mut staging_name = exe_name.to_os_string();
    staging_name.push(".update-new");
    let staging = target_exe.with_file_name(staging_name);
    let _ = std::fs::remove_file(&staging);
    std::fs::copy(&new_bin, &staging).map_err(|e| format!("Failed to stage new binary: {}", e))?;
    std::fs::set_permissions(&staging, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to mark new binary executable: {}", e))?;
    if let Err(e) = std::fs::rename(&staging, target_exe) {
        let _ = std::fs::remove_file(&staging);
        return Err(format!("Failed to replace {}: {}", target_exe.display(), e));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn launch_downloaded_installer(path: &Path) -> Result<(), String> {
    let is_msi = path.extension().map_or(false, |e| e.eq_ignore_ascii_case("msi"));
    let mut cmd = if is_msi {
        let mut c = std::process::Command::new("msiexec");
        c.arg("/i").arg(path);
        c
    } else {
        std::process::Command::new(path)
    };
    cmd.spawn().map(|_| ()).map_err(|e| format!("Failed to launch installer: {}", e))
}

#[cfg(target_os = "macos")]
fn launch_downloaded_installer(path: &Path) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to open disk image: {}", e))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn launch_downloaded_installer(path: &Path) -> Result<(), String> {
    Err(format!("No installer handler for {} on this platform", path.display()))
}

/// Check the configured GitHub repository for a release newer than this build.
#[tauri::command]
pub async fn check_for_app_update() -> Result<AppUpdateInfo, String> {
    let repo_slug = resolve_update_repo_slug();
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let client = update_http_client(std::time::Duration::from_secs(20))?;
    let release = fetch_latest_release(&client, &repo_slug).await?;

    let asset = select_platform_asset(&release.assets);
    let (install_mode, install_mode_reason) = match plan_install(asset) {
        InstallPlan::SelfReplace(_) => ("self_replace", None),
        InstallPlan::Installer => ("installer", None),
        InstallPlan::ReleasePage(reason) => ("release_page", Some(reason)),
    };

    Ok(AppUpdateInfo {
        update_available: is_newer_version(&release.tag_name, &current_version),
        current_version,
        latest_version: display_version(&release.tag_name),
        repo_slug,
        release_url: release.html_url.clone(),
        release_notes: release.body.clone().filter(|b| !b.trim().is_empty()),
        download_url: asset.map(|a| a.browser_download_url.clone()),
        release_name: release.name.clone().filter(|n| !n.trim().is_empty()),
        published_at: release.published_at.clone(),
        asset_name: asset.map(|a| a.name.clone()),
        install_mode: install_mode.to_string(),
        install_mode_reason,
    })
}

/// Apply the latest release. The release is re-fetched from the resolved repo
/// (never trusting a URL from the frontend), its asset is SHA-256 verified
/// against SHA256SUMS.txt when published, then:
///   * Linux (user-local install): binary swapped in place → restart required
///   * Windows: setup/msi launched, app exits so files can be replaced
///   * macOS: dmg opened for drag-to-Applications
///   * otherwise / `prefer_release_page`: GitHub release page opened
#[tauri::command]
pub async fn install_app_update(
    app: tauri::AppHandle,
    prefer_release_page: Option<bool>,
) -> Result<AppUpdateInstallResult, String> {
    let repo_slug = resolve_update_repo_slug();
    let api_client = update_http_client(std::time::Duration::from_secs(20))?;
    let release = fetch_latest_release(&api_client, &repo_slug).await?;
    let latest_version = display_version(&release.tag_name);

    let mut result = AppUpdateInstallResult {
        action: "opened_release_page".to_string(),
        message: String::new(),
        latest_version: latest_version.clone(),
        restart_required: false,
        will_exit: false,
        checksum_verified: false,
        path: None,
    };

    if prefer_release_page.unwrap_or(false) {
        open_external_github_url(&release.html_url)?;
        result.message = format!("Opened the v{} release page in your browser", latest_version);
        return Ok(result);
    }

    if !is_newer_version(&release.tag_name, env!("CARGO_PKG_VERSION")) {
        result.action = "up_to_date".to_string();
        result.message = format!("Rhystic Tracker v{} is already up to date", env!("CARGO_PKG_VERSION"));
        return Ok(result);
    }

    let asset = select_platform_asset(&release.assets);
    let plan = plan_install(asset);
    let asset = match (asset, &plan) {
        (Some(asset), InstallPlan::SelfReplace(_) | InstallPlan::Installer) => asset,
        (_, InstallPlan::ReleasePage(reason)) => {
            open_external_github_url(&release.html_url)?;
            result.message = format!("{} — opened the v{} release page instead", reason, latest_version);
            return Ok(result);
        }
        (None, _) => {
            open_external_github_url(&release.html_url)?;
            result.message = format!("Opened the v{} release page in your browser", latest_version);
            return Ok(result);
        }
    };

    let expected_sha = expected_asset_sha256(&api_client, &release, &asset.name).await?;
    let download_client = update_http_client(std::time::Duration::from_secs(15 * 60))?;
    let file_name = sanitize_asset_file_name(&asset.name);
    let work_dir = std::env::temp_dir().join(format!("rhystic-tracker-update-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work_dir);
    std::fs::create_dir_all(&work_dir).map_err(|e| format!("Cannot create {}: {}", work_dir.display(), e))?;

    // macOS: keep the dmg somewhere the user can find it again.
    let download_path = if cfg!(target_os = "macos") {
        dirs::download_dir().unwrap_or_else(|| work_dir.clone()).join(&file_name)
    } else {
        work_dir.join(&file_name)
    };

    let actual_sha = match download_update_asset(&download_client, &asset.browser_download_url, &download_path).await {
        Ok(sha) => sha,
        Err(e) => {
            let _ = std::fs::remove_file(&download_path);
            let _ = std::fs::remove_dir_all(&work_dir);
            return Err(e);
        }
    };
    if let Some(expected) = &expected_sha {
        if expected != &actual_sha {
            let _ = std::fs::remove_file(&download_path);
            let _ = std::fs::remove_dir_all(&work_dir);
            return Err(format!(
                "Checksum mismatch for {} (expected {}, got {}) — update aborted",
                asset.name, expected, actual_sha
            ));
        }
    }
    result.checksum_verified = expected_sha.is_some();

    match plan {
        InstallPlan::SelfReplace(target_exe) => {
            #[cfg(target_os = "linux")]
            {
                let (archive, dir, target) = (download_path.clone(), work_dir.clone(), target_exe.clone());
                let applied = tokio::task::spawn_blocking(move || linux_apply_update_tarball(&archive, &dir, &target))
                    .await
                    .map_err(|e| format!("Update task failed: {}", e))
                    .and_then(|r| r);
                let _ = std::fs::remove_dir_all(&work_dir);
                applied?;
                result.action = "binary_replaced".to_string();
                result.restart_required = true;
                result.path = Some(target_exe.to_string_lossy().to_string());
                result.message = format!(
                    "Installed v{} to {} — restart Rhystic Tracker to finish updating",
                    latest_version,
                    target_exe.display()
                );
            }
            #[cfg(not(target_os = "linux"))]
            {
                let _ = (target_exe, &work_dir);
                return Err("In-place binary updates are only supported on Linux".to_string());
            }
        }
        InstallPlan::Installer => {
            launch_downloaded_installer(&download_path)?;
            result.action = "installer_launched".to_string();
            result.path = Some(download_path.to_string_lossy().to_string());
            if cfg!(target_os = "windows") {
                // Exit so the installer can overwrite the running executable.
                result.will_exit = true;
                result.message = format!("Launched the v{} installer — Rhystic Tracker will now close", latest_version);
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    handle.exit(0);
                });
            } else {
                result.message = format!(
                    "Opened the v{} disk image — drag Rhystic Tracker into Applications, then restart",
                    latest_version
                );
            }
        }
        InstallPlan::ReleasePage(_) => unreachable!("release-page plans return early"),
    }

    let _ = &app;
    Ok(result)
}

/// Relaunch the app (used after an in-place update). Goes through Tauri's exit
/// path so the single-instance lock is released before the new process starts.
#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.request_restart();
}

#[cfg(test)]
mod update_checker_tests {
    use super::*;

    #[test]
    fn parses_github_slugs() {
        let ok = |raw: &str, bare: bool| parse_github_slug(raw, bare);
        assert_eq!(ok("https://github.com/Matt4C6F656872/Rhystic-Tracker-Web", false).as_deref(), Some("Matt4C6F656872/Rhystic-Tracker-Web"));
        assert_eq!(ok("https://github.com/Balthazzahr/Rhystic-Tracker.git/", false).as_deref(), Some("Balthazzahr/Rhystic-Tracker"));
        assert_eq!(ok("git@github.com:owner/repo.git", false).as_deref(), Some("owner/repo"));
        assert_eq!(ok("github.com/owner/repo/tree/main", false).as_deref(), Some("owner/repo"));
        assert_eq!(ok("owner/repo", true).as_deref(), Some("owner/repo"));
        assert_eq!(ok("owner/repo", false), None);
        assert_eq!(ok("", false), None);
        assert_eq!(ok("https://gitlab.com/owner/repo", false), None);
        assert_eq!(ok("https://github.com.evil.example/owner/repo", false), None);
        assert_eq!(ok("https://github.com/owner", false), None);
        assert_eq!(ok("https://github.com/../repo", false), None);
    }

    #[test]
    fn resolves_repo_from_cargo_metadata() {
        // Cargo.toml sets `repository`; with no RHYSTIC_UPDATE_REPO override the
        // slug must come from it rather than the upstream fallback.
        if option_env!("RHYSTIC_UPDATE_REPO").is_none() {
            let expected = parse_github_slug(env!("CARGO_PKG_REPOSITORY"), false)
                .unwrap_or_else(|| DEFAULT_UPDATE_REPO.to_string());
            assert_eq!(resolve_update_repo_slug(), expected);
        }
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer_version("v2.1.0", "2.0.0"));
        assert!(is_newer_version("2.0.1", "2.0.0"));
        assert!(is_newer_version("v3", "2.9.9"));
        assert!(is_newer_version("2.1.0", "2.1.0-beta.1"));
        assert!(!is_newer_version("v2.0.0", "2.0.0"));
        assert!(!is_newer_version("v1.8.0", "2.0.0"));
        assert!(!is_newer_version("2.1.0-beta", "2.1.0"));
        assert!(!is_newer_version("nightly", "2.0.0"));
        assert_eq!(display_version("v2.1.0"), "2.1.0");
        assert_eq!(display_version("rhystic-tracker-v1.8.0"), "1.8.0");
    }

    #[test]
    fn sanitizes_asset_names() {
        assert_eq!(sanitize_asset_file_name("rhystic-tracker-linux-x86_64.tar.gz"), "rhystic-tracker-linux-x86_64.tar.gz");
        assert_eq!(sanitize_asset_file_name("../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_asset_file_name("..."), "rhystic-tracker-update.bin");
    }

    #[test]
    fn selects_platform_asset() {
        let assets: Vec<GithubReleaseAsset> = [
            "rhystic-tracker-linux-x86_64.tar.gz",
            "rhystic-tracker-macos-universal.dmg",
            "rhystic-tracker-windows-setup.exe",
            "rhystic-tracker-windows-x86_64.zip",
            "SHA256SUMS.txt",
        ]
        .iter()
        .map(|n| GithubReleaseAsset { name: n.to_string(), browser_download_url: format!("https://github.com/o/r/releases/download/v1/{}", n) })
        .collect();
        let picked = select_platform_asset(&assets).map(|a| a.name.as_str());
        if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            assert_eq!(picked, Some("rhystic-tracker-linux-x86_64.tar.gz"));
        } else if cfg!(target_os = "macos") {
            assert_eq!(picked, Some("rhystic-tracker-macos-universal.dmg"));
        } else if cfg!(target_os = "windows") {
            assert_eq!(picked, Some("rhystic-tracker-windows-setup.exe"));
        }
    }
}
