//! Discover installed applications on the host.
//!
//! macOS: scan `.app` bundles under standard locations + extract icons via
//! `sips` (concurrent, semaphore-limited).
//! Windows: single PowerShell invocation that enumerates Start Menu `.lnk`
//! shortcuts via `WScript.Shell` (COM) and extracts icons via
//! `System.Drawing` in the same script. One subprocess = fast and reliable.
//! Linux: stub (no apps surfaced; RunCommand action lands separately).

use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[allow(unused)]
use base64::Engine;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub icon_base64: Option<String>,
}

pub async fn discover() -> Result<Vec<InstalledApp>> {
    #[cfg(target_os = "macos")]
    {
        macos::discover().await
    }
    #[cfg(target_os = "windows")]
    {
        windows::discover().await
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(Vec::new())
    }
}

// =====================================================================
// macOS
// =====================================================================

#[cfg(target_os = "macos")]
mod macos {
    use super::{InstalledApp, Path, PathBuf};
    use anyhow::Result;
    use base64::Engine;
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    const MAX_CONCURRENT_ICON_DECODES: usize = 8;
    const ICON_TARGET_SIZE: u32 = 128;
    const MAX_DEPTH: usize = 2;

    pub async fn discover() -> Result<Vec<InstalledApp>> {
        let mut bare = Vec::new();
        for dir in scan_roots() {
            if dir.exists() {
                scan_dir(&dir, &mut bare, 0);
            }
        }

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
}

// =====================================================================
// Windows — single PowerShell invocation
// =====================================================================

#[cfg(target_os = "windows")]
mod windows {
    use super::InstalledApp;
    use anyhow::Result;

    /// Enumerates Start Menu `.lnk` shortcuts via WScript.Shell COM,
    /// resolves their targets, extracts associated icons as base64 PNG,
    /// and emits one record per line in the format:
    ///   `APP|<name>|<target_path>|<icon_base64>`
    /// Lines that fail at any step are silently skipped.
    const DISCOVERY_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
Add-Type -AssemblyName System.Drawing 2>$null

$shell = New-Object -ComObject WScript.Shell
$menus = @(
    [System.IO.Path]::Combine($env:APPDATA, 'Microsoft\Windows\Start Menu\Programs'),
    [System.IO.Path]::Combine($env:PROGRAMDATA, 'Microsoft\Windows\Start Menu\Programs')
)

$seen = @{}
foreach ($menu in $menus) {
    if (-not (Test-Path $menu)) { continue }
    Get-ChildItem -Path $menu -Recurse -Filter '*.lnk' -ErrorAction SilentlyContinue | ForEach-Object {
        try {
            $sc = $shell.CreateShortcut($_.FullName)
            $target = $sc.TargetPath
            if ([string]::IsNullOrEmpty($target)) { return }
            if (-not $target.ToLower().EndsWith('.exe')) { return }
            if ($seen.ContainsKey($target)) { return }
            $seen[$target] = $true

            $b64 = ''
            try {
                $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($target)
                if ($icon -ne $null) {
                    $bmp = $icon.ToBitmap()
                    $resized = New-Object System.Drawing.Bitmap($bmp, 128, 128)
                    $ms = New-Object System.IO.MemoryStream
                    $resized.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
                    $b64 = [Convert]::ToBase64String($ms.ToArray())
                    $ms.Dispose(); $resized.Dispose(); $bmp.Dispose(); $icon.Dispose()
                }
            } catch { }

            $name = [System.IO.Path]::GetFileNameWithoutExtension($_.FullName)
            Write-Host ('APP|' + $name + '|' + $target + '|' + $b64)
        } catch { }
    }
}
"#;

    pub async fn discover() -> Result<Vec<InstalledApp>> {
        let output = tokio::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                DISCOVERY_SCRIPT,
            ])
            .output()
            .await?;

        if !output.status.success() {
            tracing::warn!(
                stderr = %String::from_utf8_lossy(&output.stderr),
                "windows app discovery failed"
            );
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut apps = Vec::new();
        for line in stdout.lines() {
            let Some(rest) = line.strip_prefix("APP|") else { continue };
            let mut parts = rest.splitn(3, '|');
            let (Some(name), Some(path), Some(icon)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            apps.push(InstalledApp {
                name: name.to_string(),
                path: path.to_string(),
                icon_base64: if icon.is_empty() {
                    None
                } else {
                    Some(icon.to_string())
                },
            });
        }
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(apps)
    }
}
