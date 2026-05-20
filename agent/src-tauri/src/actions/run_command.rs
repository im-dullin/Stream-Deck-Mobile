//! Spawn an arbitrary process. Detached (fire-and-forget): the action returns
//! as soon as the child is spawned, and a background task reaps the exit
//! status so we don't leave zombies. Long-running scripts (crawlers, image
//! pipelines) don't block subsequent sub-actions in a multi-action.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::process::Command;

pub async fn run(
    program: &str,
    args: &[String],
    working_dir: Option<&str>,
) -> Result<()> {
    let program = expand_tilde(program);
    let expanded_args: Vec<String> = args.iter().map(|a| expand_tilde(a)).collect();
    let wd = working_dir.map(expand_tilde);

    tracing::info!(
        program = %program,
        args = ?expanded_args,
        working_dir = ?wd,
        "run_command"
    );

    let mut cmd = Command::new(&program);
    cmd.args(&expanded_args);
    if let Some(dir) = wd.as_deref() {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("spawn `{}`", program))?;

    // Reap the child in the background so we don't accumulate zombies on Unix.
    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) if !status.success() => {
                tracing::warn!(?status, "run_command exited with non-zero status");
            }
            Err(e) => tracing::warn!(error = ?e, "run_command wait failed"),
            _ => {}
        }
    });

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
