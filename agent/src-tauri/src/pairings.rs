//! Per-device pairings persistence. Each device that successfully completes
//! the approval flow gets its own token stored here; subsequent connections
//! validate against this list. Tokens can be revoked individually by removing
//! the matching device.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Pairing {
    pub device_id: String,
    pub device_name: String,
    pub token: String,
    pub paired_at_unix: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingDb {
    pub devices: Vec<Pairing>,
}

impl PairingDb {
    pub fn find(&self, device_id: &str, token: &str) -> Option<&Pairing> {
        self.devices
            .iter()
            .find(|p| p.device_id == device_id && p.token == token)
    }

    pub fn upsert(&mut self, pairing: Pairing) {
        self.devices.retain(|p| p.device_id != pairing.device_id);
        self.devices.push(pairing);
    }

    pub fn remove(&mut self, device_id: &str) -> bool {
        let before = self.devices.len();
        self.devices.retain(|p| p.device_id != device_id);
        before != self.devices.len()
    }
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn pairings_path() -> Result<PathBuf> {
    Ok(crate::config::config_dir()?.join("pairings.json"))
}

pub async fn load_or_init() -> Result<PairingDb> {
    let path = pairings_path()?;
    if path.exists() {
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("read {}", path.display()))?;
        Ok(serde_json::from_slice(&bytes).context("parse pairings.json")?)
    } else {
        Ok(PairingDb::default())
    }
}

pub async fn save(db: &PairingDb) -> Result<()> {
    let path = pairings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(db)?;
    fs::write(&tmp, &json)
        .await
        .with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, &path)
        .await
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}
