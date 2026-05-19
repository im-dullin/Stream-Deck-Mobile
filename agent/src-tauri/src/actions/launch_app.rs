use anyhow::{Context, Result, bail};
use tokio::process::Command;

pub async fn run(app_path: &str, app_name: &str) -> Result<()> {
    tracing::info!(app_name, app_path, "launch_app");

    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(app_path);
        c
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", app_path]);
        c
    };

    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(app_path);
        c
    };

    let status = cmd
        .status()
        .await
        .with_context(|| format!("spawn launcher for {}", app_name))?;

    if !status.success() {
        bail!("launcher exited with {} for {}", status, app_name);
    }
    Ok(())
}
