//! Action execution.
//!
//! `execute` is the entry point. It handles `MultiAction` by iterating over
//! the sub-actions sequentially via `execute_single`. Errors in one sub-action
//! are logged but do not abort the remaining ones (Stream Deck convention).
//!
//! Adding a new action type:
//!   1. Add a variant in `protocol::Action`
//!   2. Add a sibling module here with a `run(...)` function
//!   3. Add a match arm in `execute_single`

use crate::protocol::Action;
use anyhow::Result;

pub mod launch_app;
pub mod open_url;

pub async fn execute(action: &Action) -> Result<()> {
    match action {
        Action::MultiAction { actions } => {
            for (i, sub) in actions.iter().enumerate() {
                if let Err(e) = execute_single(sub).await {
                    tracing::error!(index = i, error = ?e, "sub-action failed; continuing");
                }
            }
            Ok(())
        }
        single => execute_single(single).await,
    }
}

async fn execute_single(action: &Action) -> Result<()> {
    match action {
        Action::LaunchApp { app_path, app_name } => {
            launch_app::run(app_path, app_name).await
        }
        Action::OpenUrl { url, .. } => open_url::run(url).await,
        Action::MultiAction { .. } => {
            // Nested multi-actions are conceptually allowed by the schema but
            // the editor UI keeps the structure flat; flatten defensively.
            tracing::warn!("nested multi-action skipped (use a flat list of single actions)");
            Ok(())
        }
    }
}
