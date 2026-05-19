//! Persist the active profile to disk. Atomic write via tmp+rename so a
//! crash mid-save cannot leave a corrupt JSON.

use crate::protocol::{Page, Profile};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

const APP_DIR: &str = "StreamDeckVirtual";
const PROFILE_FILE: &str = "profile.json";

pub fn config_dir() -> Result<PathBuf> {
    let base = dirs::data_dir()
        .or_else(dirs::config_dir)
        .context("could not resolve user data directory")?;
    Ok(base.join(APP_DIR))
}

pub fn profile_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(PROFILE_FILE))
}

pub async fn load_or_init() -> Result<Profile> {
    let path = profile_path()?;
    if path.exists() {
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("read {}", path.display()))?;
        let profile: Profile =
            serde_json::from_slice(&bytes).context("parse profile.json")?;
        Ok(profile)
    } else {
        let profile = default_profile();
        save(&profile).await?;
        Ok(profile)
    }
}

pub async fn save(profile: &Profile) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("mkdir {}", dir.display()))?;
    let path = profile_path()?;
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(profile)?;
    fs::write(&tmp, &json)
        .await
        .with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, &path).await.with_context(|| {
        format!("rename {} -> {}", tmp.display(), path.display())
    })?;
    Ok(())
}

fn default_profile() -> Profile {
    Profile {
        id: "default".into(),
        name: "Default".into(),
        default_page_id: "p1".into(),
        pages: vec![Page {
            id: "p1".into(),
            name: "Page 1".into(),
            rows: 3,
            cols: 5,
            buttons: vec![],
        }],
    }
}
