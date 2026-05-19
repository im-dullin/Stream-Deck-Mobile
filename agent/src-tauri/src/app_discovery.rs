//! Discover installed applications on the host.
//!
//! macOS: scan .app bundles under standard locations + extract icons via `sips`.
//! Windows: parse Start Menu .lnk shortcuts via the `lnk` crate, resolve target
//!   executables, extract icons via PowerShell's System.Drawing.
//! Linux: stub (no apps surfaced; users can map via RunCommand once added).
//!
//! All icon work runs concurrently behind a semaphore (8 in flight) so a
//! machine with 100+ apps still finishes in ~1s.

use anyhow::Result;
use base64::Engine;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const MAX_CONCURRENT_ICON_DECODES: usize = 8;
const ICON_TARGET_SIZE: u32 = 128;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub icon_base64: Option<String>,
}

pub async fn discover() -> Result<Vec<InstalledApp>> {
    let bare = scan_apps();

    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_ICON_DECODES));
    let mut tasks: JoinSet<InstalledApp> = JoinSet::new();
    for app in bare {
        let sem = sem.clone();
        tasks.spawn(async move {
            let _permit = sem.acquire_owned().await.expect("semaphore closed");
            let icon = extract_icon(Path::new(&app.path)).await;
            InstalledApp { icon_base64: icon, ..app }
        });
    }

    let mut apps = Vec::new();
    while let Some(res) = tasks.join_next().await {
        if let Ok(app) = res {
            apps.push(app);
        }
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.path == b.path);
    Ok(apps)
}

fn scan_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    for dir in scan_roots() {
        if dir.exists() {
            scan_dir(&dir, &mut apps, 0);
        }
    }
    apps
}

// =====================================================================
// macOS
// =====================================================================

#[cfg(target_os = "macos")]
fn scan_roots() -> Vec<PathBuf> {
    let mut v = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Some(home) = dirs::home_dir() {
        v.push(home.join("Applications"));
    }
    v
}

#[cfg(target_os = "macos")]
const MAX_DEPTH: usize = 2;

#[cfg(target_os = "macos")]
fn scan_dir(dir: &Path, apps: &mut Vec<InstalledApp>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if name.starts_with('.') {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some("app") {
            let display_name = name.trim_end_matches(".app").to_string();
            if let Some(path_str) = path.to_str() {
                apps.push(InstalledApp {
                    name: display_name,
                    path: path_str.to_string(),
                    icon_base64: None,
                });
            }
        } else if path.is_dir() {
            scan_dir(&path, apps, depth + 1);
        }
    }
}

#[cfg(target_os = "macos")]
async fn extract_icon(app_path: &Path) -> Option<String> {
    let icns = find_icns(app_path)?;
    let tmp = std::env::temp_dir()
        .join(format!("sd-icon-{}.png", uuid::Uuid::new_v4().simple()));
    let output = tokio::process::Command::new("sips")
        .args(["-s", "format", "png", "-Z", &ICON_TARGET_SIZE.to_string()])
        .arg(&icns)
        .arg("--out")
        .arg(&tmp)
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = tokio::fs::read(&tmp).await.ok()?;
    let _ = tokio::fs::remove_file(&tmp).await;
    Some(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[cfg(target_os = "macos")]
fn find_icns(app_path: &Path) -> Option<PathBuf> {
    let resources = app_path.join("Contents/Resources");
    let app_basename = app_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let entries = std::fs::read_dir(&resources).ok()?;
    let mut candidates: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("icns"))
        .collect();
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by_key(|p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem == app_basename {
            0
        } else if stem == "AppIcon" {
            1
        } else {
            2
        }
    });
    candidates.into_iter().next()
}

// =====================================================================
// Windows
// =====================================================================

#[cfg(target_os = "windows")]
fn scan_roots() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        v.push(PathBuf::from(format!(
            r"{}\Microsoft\Windows\Start Menu\Programs",
            appdata
        )));
    }
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        v.push(PathBuf::from(format!(
            r"{}\Microsoft\Windows\Start Menu\Programs",
            programdata
        )));
    }
    v
}

#[cfg(target_os = "windows")]
const MAX_DEPTH: usize = 5;

#[cfg(target_os = "windows")]
fn scan_dir(dir: &Path, apps: &mut Vec<InstalledApp>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_dir(&path, apps, depth + 1);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()).map(str::to_lowercase) == Some("lnk".into()) {
            if let Some((display, target)) = parse_lnk(&path) {
                let lower = target
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_lowercase);
                if lower.as_deref() != Some("exe") {
                    continue;
                }
                if let Some(target_str) = target.to_str() {
                    apps.push(InstalledApp {
                        name: display,
                        path: target_str.to_string(),
                        icon_base64: None,
                    });
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn parse_lnk(path: &Path) -> Option<(String, PathBuf)> {
    let shortcut = lnk::ShellLink::open(path).ok()?;
    let target = shortcut
        .link_info()
        .as_ref()
        .and_then(|li| li.local_base_path().as_ref().map(|s| s.to_string()))?;
    let display = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())?;
    Some((display, PathBuf::from(target)))
}

#[cfg(target_os = "windows")]
async fn extract_icon(target_path: &Path) -> Option<String> {
    let Some(target_str) = target_path.to_str() else { return None };
    let tmp = std::env::temp_dir().join(format!(
        "sd-icon-{}.png",
        uuid::Uuid::new_v4().simple()
    ));
    let Some(tmp_str) = tmp.to_str() else { return None };

    // Inline PowerShell — extracts the associated icon and resizes to 128×128 PNG.
    let script = format!(
        r#"$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon('{target}')
$bmp = $icon.ToBitmap()
$resized = New-Object System.Drawing.Bitmap($bmp, {size}, {size})
$resized.Save('{out}', [System.Drawing.Imaging.ImageFormat]::Png)
$resized.Dispose(); $bmp.Dispose(); $icon.Dispose()"#,
        target = target_str.replace('\'', "''"),
        out = tmp_str.replace('\'', "''"),
        size = ICON_TARGET_SIZE,
    );

    let output = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = tokio::fs::read(&tmp).await.ok()?;
    let _ = tokio::fs::remove_file(&tmp).await;
    Some(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

// =====================================================================
// Linux (stub)
// =====================================================================

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn scan_roots() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const MAX_DEPTH: usize = 0;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn scan_dir(_dir: &Path, _apps: &mut Vec<InstalledApp>, _depth: usize) {}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
async fn extract_icon(_p: &Path) -> Option<String> {
    None
}
