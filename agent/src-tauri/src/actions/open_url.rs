use anyhow::{bail, Context, Result};
use tokio::process::Command;

pub async fn run(url: &str) -> Result<()> {
    tracing::info!(url, "open_url");

    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(url);
        c
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    };

    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(url);
        c
    };

    let status = cmd
        .status()
        .await
        .with_context(|| format!("spawn opener for {}", url))?;

    if !status.success() {
        bail!("opener exited with {} for {}", status, url);
    }
    Ok(())
}
