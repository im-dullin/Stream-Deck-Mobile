//! Reveal a directory in the host's file manager. Behavior:
//!   - macOS: `open <path>` — pops up Finder at that location.
//!   - Windows: `explorer.exe <path>` — opens Explorer.
//!   - Linux: `xdg-open <path>` — defers to the user's default file manager.
//!
//! `~/` is expanded to the user's home on the agent side so users can store
//! portable paths like `~/Documents/회의록`.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tokio::process::Command;

pub async fn run(path: &str) -> Result<()> {
    let path = expand_tilde(path);
    tracing::info!(path = %path, "open_folder");

    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(&path);
        c
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("explorer.exe");
        c.arg(&path);
        c
    };

    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(&path);
        c
    };

    let status = cmd
        .status()
        .await
        .with_context(|| format!("spawn folder opener for {}", path))?;

    // `explorer.exe` returns 1 on success in many cases. Don't treat that as
    // an error on Windows; everywhere else a non-zero status really is one.
    #[cfg(not(target_os = "windows"))]
    if !status.success() {
        bail!("folder opener exited with {} for {}", status, path);
    }
    #[cfg(target_os = "windows")]
    let _ = status;

    Ok(())
}

fn expand_tilde(s: &str) -> String {
    if s == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().into_owned();
        }
    }
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            let joined: PathBuf = home.join(rest);
            return joined.to_string_lossy().into_owned();
        }
    }
    s.to_string()
}
