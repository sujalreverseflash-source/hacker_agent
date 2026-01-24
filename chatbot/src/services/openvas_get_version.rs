use anyhow::Result;
use serde_json::Value;

use crate::api::openvas;

/// Business-logic layer for "OpenVAS get version" using the Go backend.
/// Right now this is just a thin wrapper, but we can later add parsing
/// or normalization (e.g. extract only the numeric version).
pub async fn openvas_get_version() -> Result<Value> {
    openvas::get_version().await
}

